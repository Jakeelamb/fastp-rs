//! End-to-end preprocessing: single-end, paired-end, interleaved, merge, split, QC reports.

use crate::complexity::passes_low_complexity;
use crate::config::RunConfig;
use crate::dedup::{fingerprint48, fingerprint_pair, DedupGate};
use crate::fastq::FastqReadMut;
use crate::fastq::OwnedFastqRead;
use crate::io::open_fastq_reader;
use crate::io::open_fastq_writer;
use crate::io::path_is_stdio_dash;
use crate::io::FastqWriter;
use crate::io::ReaderOptions;
use crate::io::WriterOptions;
use crate::merge::apply_pe_overlap_correction;
use crate::merge::try_merge_pe;
use crate::phred::convert_qual_phred64_to_33;
use crate::qc::QcCollector;
use crate::quality_gate::read_passes_quality_filters;
use crate::read_parts::MutableRead;
use crate::report;
use crate::split::part_path;
use crate::trim::apply_all_trims;
use crate::umi::apply_umi;
use crate::umi::apply_umi_single_end;
use crate::Error;
use crate::Result;
use std::io::Write;

/// Counters returned after a successful `run`.
#[derive(Debug, Default, Clone)]
pub struct RunStats {
    pub reads_in: u64,
    pub reads_out: u64,
    pub pairs_in: u64,
    pub pairs_out: u64,
    pub merged_pairs: u64,
}

struct RotatingWriter {
    base_path: std::path::PathBuf,
    split_every: Option<u64>,
    split_digits: Option<u8>,
    writer_opts: WriterOptions,
    part: u32,
    written_in_part: u64,
    writer: FastqWriter,
}

impl RotatingWriter {
    fn create(
        base_path: std::path::PathBuf,
        split_every: Option<u64>,
        split_digits: Option<u8>,
        writer_opts: WriterOptions,
    ) -> Result<Self> {
        let part = 1;
        let path = if split_every.is_some() {
            part_path(&base_path, part, split_digits)
        } else {
            base_path.clone()
        };
        let writer = open_fastq_writer(&path, writer_opts)?;
        Ok(Self {
            base_path,
            split_every,
            split_digits,
            writer_opts,
            part,
            written_in_part: 0,
            writer,
        })
    }

    fn rotate_if_needed(&mut self) -> Result<()> {
        let Some(limit) = self.split_every else {
            return Ok(());
        };
        if self.written_in_part < limit {
            return Ok(());
        }
        self.writer.flush()?;
        self.part += 1;
        self.written_in_part = 0;
        let path = part_path(&self.base_path, self.part, self.split_digits);
        self.writer = open_fastq_writer(&path, self.writer_opts)?;
        Ok(())
    }

    fn write_record(&mut self, rec: &OwnedFastqRead) -> Result<()> {
        self.rotate_if_needed()?;
        rec.write_to(&mut self.writer)?;
        self.written_in_part += 1;
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

/// Paired-end output: two files, or interleaved stdout (`-`, plain or gzip per `WriterOptions`).
enum PeOut {
    Files {
        o1: Box<RotatingWriter>,
        o2: Box<RotatingWriter>,
    },
    Stdout(FastqWriter),
}

impl PeOut {
    fn create(cfg: &RunConfig, writer_opts: WriterOptions) -> Result<Self> {
        if path_is_stdio_dash(&cfg.out1) {
            return Ok(Self::Stdout(open_fastq_writer(&cfg.out1, writer_opts)?));
        }
        let o2 = cfg.out2.clone().ok_or_else(|| {
            Error::Config("paired mode requires -O/--out2 when not using stdout (-o -)".into())
        })?;
        Ok(Self::Files {
            o1: Box::new(RotatingWriter::create(
                cfg.out1.clone(),
                cfg.split_reads_per_file,
                cfg.split_prefix_digits,
                writer_opts,
            )?),
            o2: Box::new(RotatingWriter::create(
                o2,
                cfg.split_reads_per_file,
                cfg.split_prefix_digits,
                writer_opts,
            )?),
        })
    }

    fn write_pair(&mut self, a: &OwnedFastqRead, b: &OwnedFastqRead) -> Result<()> {
        match self {
            Self::Files { o1, o2 } => {
                o1.write_record(a)?;
                o2.write_record(b)?;
            }
            Self::Stdout(w) => {
                a.write_to(&mut *w)?;
                b.write_to(&mut *w)?;
            }
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        match self {
            Self::Files { o1, o2 } => {
                o1.finish()?;
                o2.finish()?;
            }
            Self::Stdout(w) => {
                w.flush()?;
            }
        }
        Ok(())
    }
}

fn writer_opts_from_cfg(cfg: &RunConfig) -> WriterOptions {
    WriterOptions {
        gzip_level: cfg.gzip_compression_level,
        stdout_gzip: cfg.stdout_gzip,
    }
}

fn read_opts_from_cfg(cfg: &RunConfig) -> ReaderOptions {
    ReaderOptions {
        stdin_gzip: cfg.stdin_gzip,
    }
}

fn maybe_phred64(seq_len: usize, qual: &mut [u8], phred64: bool) {
    if phred64 && qual.len() == seq_len {
        convert_qual_phred64_to_33(qual);
    }
}

fn read_passes_output_filters(r: &MutableRead, cfg: &RunConfig) -> bool {
    if r.is_empty() {
        return false;
    }
    if !passes_low_complexity(&r.seq, cfg.low_complexity_filter, cfg.complexity_threshold) {
        return false;
    }
    if !cfg.disable_length_filtering {
        if let Some(min) = cfg.length_required {
            if r.len() < min {
                return false;
            }
        }
        if let Some(max) = cfg.length_limit {
            if r.len() > max {
                return false;
            }
        }
    }
    read_passes_quality_filters(&r.seq, &r.qual, cfg)
}

fn passes_len(len: usize, cfg: &RunConfig) -> bool {
    if len == 0 {
        return false;
    }
    if cfg.disable_length_filtering {
        return true;
    }
    if let Some(min) = cfg.length_required {
        if len < min {
            return false;
        }
    }
    if let Some(max) = cfg.length_limit {
        if len > max {
            return false;
        }
    }
    true
}

/// Run full pipeline according to `cfg`.
pub fn run(cfg: &RunConfig) -> Result<RunStats> {
    cfg.validate().map_err(Error::Config)?;

    let mut qc = QcCollector::new(!cfg.dont_eval_duplication);
    let mut stats = RunStats::default();

    if cfg.is_paired() {
        run_paired(cfg, &mut qc, &mut stats)?;
    } else {
        run_single_end(cfg, &mut qc, &mut stats)?;
    }

    if let Some(ref p) = cfg.json_report {
        report::write_json_report(p, &qc, stats.reads_in, stats.reads_out)?;
    }
    if let Some(ref p) = cfg.html_report {
        report::write_html_report(
            p,
            &qc,
            stats.reads_in,
            stats.reads_out,
            cfg.report_title.as_deref(),
        )?;
    }

    Ok(stats)
}

fn run_single_end(cfg: &RunConfig, qc: &mut QcCollector, stats: &mut RunStats) -> Result<()> {
    let ro = read_opts_from_cfg(cfg);
    let wo = writer_opts_from_cfg(cfg);
    let mut reader = open_fastq_reader(&cfg.read1, ro)?;
    let mut out = RotatingWriter::create(
        cfg.out1.clone(),
        cfg.split_reads_per_file,
        cfg.split_prefix_digits,
        wo,
    )?;
    let mut failed_w = if let Some(ref p) = cfg.failed_out {
        Some(open_fastq_writer(p, wo)?)
    } else {
        None
    };
    let mut rec = FastqReadMut::default();
    let mut global_line = 0usize;
    let mut dedup_gate = if cfg.dedup {
        Some(DedupGate::default())
    } else {
        None
    };

    while let Some(start) = rec.read_four_lines(&mut reader, &mut global_line)? {
        if let Some(limit) = cfg.reads_to_process {
            if stats.reads_in >= limit {
                break;
            }
        }
        rec.validate(start)?;
        stats.reads_in += 1;

        let owned = rec.as_owned();
        let mut m = MutableRead::from_owned(owned)?;
        maybe_phred64(m.seq.len(), &mut m.qual, cfg.phred64);
        let fp = fingerprint48(&m.seq);
        qc.observe_before_se(&m);
        apply_umi_single_end(&mut m, cfg.umi_source, cfg);
        apply_all_trims(&mut m, cfg, false);

        if !read_passes_output_filters(&m, cfg) {
            if let Some(ref mut w) = failed_w {
                m.to_owned().write_to(w)?;
            }
            continue;
        }

        if let Some(ref mut g) = dedup_gate {
            if !g.allow_emit(fp) {
                continue;
            }
        }

        let out_rec = m.to_owned();
        qc.observe_after_se(&m);
        out.write_record(&out_rec)?;
        stats.reads_out += 1;
    }
    out.finish()?;
    if let Some(mut w) = failed_w {
        w.flush()?;
    }
    Ok(())
}

fn run_paired(cfg: &RunConfig, qc: &mut QcCollector, stats: &mut RunStats) -> Result<()> {
    let ro = read_opts_from_cfg(cfg);
    let wo = writer_opts_from_cfg(cfg);
    let mut r1_reader = open_fastq_reader(&cfg.read1, ro)?;
    let mut r2_reader = if cfg.interleaved {
        None
    } else {
        Some(open_fastq_reader(
            cfg.read2.as_ref().ok_or_else(|| {
                Error::Config("paired mode requires -I/--in2 unless --interleaved".into())
            })?,
            ro,
        )?)
    };

    let mut pe_out = PeOut::create(cfg, wo)?;

    let mut merged_out: Option<RotatingWriter> = if cfg.merge_pe {
        Some(RotatingWriter::create(
            cfg.merged_out.clone().ok_or_else(|| {
                Error::Config("--merged-out is required when --merge is set".into())
            })?,
            cfg.split_reads_per_file,
            cfg.split_prefix_digits,
            wo,
        )?)
    } else {
        None
    };

    let mut failed_w = if let Some(ref p) = cfg.failed_out {
        Some(open_fastq_writer(p, wo)?)
    } else {
        None
    };

    let mut unpaired1_w = cfg
        .unpaired1
        .as_ref()
        .map(|p| open_fastq_writer(p, wo))
        .transpose()?;
    let mut unpaired2_w = cfg
        .unpaired2
        .as_ref()
        .map(|p| open_fastq_writer(p, wo))
        .transpose()?;

    let mut dedup_gate = if cfg.dedup {
        Some(DedupGate::default())
    } else {
        None
    };

    let mut rec1 = FastqReadMut::default();
    let mut rec2 = FastqReadMut::default();
    let mut g1 = 0usize;
    let mut g2 = 0usize;

    loop {
        if let Some(limit) = cfg.reads_to_process {
            if stats.pairs_in >= limit {
                break;
            }
        }
        let (s1, s2) = if cfg.interleaved {
            let s1 = rec1.read_four_lines(&mut r1_reader, &mut g1)?;
            let s2 = rec2.read_four_lines(&mut r1_reader, &mut g1)?;
            (s1, s2)
        } else {
            let s1 = rec1.read_four_lines(&mut r1_reader, &mut g1)?;
            let s2 = rec2.read_four_lines(r2_reader.as_mut().unwrap(), &mut g2)?;
            (s1, s2)
        };

        match (s1, s2) {
            (None, None) => break,
            (Some(a), Some(b)) => {
                rec1.validate(a)?;
                rec2.validate(b)?;
            }
            _ => {
                return Err(Error::MismatchedPairEof);
            }
        }

        stats.pairs_in += 1;
        stats.reads_in += 2;

        let o1 = rec1.as_owned();
        let o2 = rec2.as_owned();
        let mut m1 = MutableRead::from_owned(o1)?;
        let mut m2 = MutableRead::from_owned(o2)?;
        maybe_phred64(m1.seq.len(), &mut m1.qual, cfg.phred64);
        maybe_phred64(m2.seq.len(), &mut m2.qual, cfg.phred64);
        let pair_fp = fingerprint_pair(&m1.seq, &m2.seq);
        qc.observe_before_pair(&m1, &m2);

        apply_umi(&mut m1, &mut m2, cfg.umi_source, cfg);
        apply_all_trims(&mut m1, cfg, false);
        apply_all_trims(&mut m2, cfg, true);

        if cfg.merge_pe {
            if cfg.pe_overlap_correction {
                apply_pe_overlap_correction(
                    &mut m1,
                    &mut m2,
                    cfg.overlap_len_min,
                    cfg.overlap_match_fraction,
                    cfg.overlap_diff_max,
                    cfg.overlap_diff_percent_limit / 100.0,
                );
            }
            if let Some(merged) = try_merge_pe(
                &m1,
                &m2,
                cfg.overlap_len_min,
                cfg.overlap_match_fraction,
                cfg.overlap_diff_max,
                cfg.overlap_diff_percent_limit / 100.0,
            ) {
                if !passes_len(merged.seq.len(), cfg) {
                    write_failed_pair(&mut failed_w, &m1, &m2)?;
                    continue;
                }
                if let Some(ref mut g) = dedup_gate {
                    if !g.allow_emit(pair_fp) {
                        continue;
                    }
                }
                let mo = merged.to_owned_fastq();
                qc.observe_after_merged(&merged.seq, &merged.qual);
                merged_out
                    .as_mut()
                    .ok_or_else(|| Error::Config("internal: merged writer missing".into()))?
                    .write_record(&mo)?;
                stats.merged_pairs += 1;
                stats.reads_out += 1;
                if cfg.merge_include_unmerged
                    && read_passes_output_filters(&m1, cfg)
                    && read_passes_output_filters(&m2, cfg)
                {
                    qc.observe_after_pair(&m1, &m2);
                    pe_out.write_pair(&m1.to_owned(), &m2.to_owned())?;
                    stats.pairs_out += 1;
                    stats.reads_out += 2;
                }
            } else {
                handle_pe_pair_emit(
                    cfg,
                    &mut pe_out,
                    &mut failed_w,
                    &mut unpaired1_w,
                    &mut unpaired2_w,
                    &mut dedup_gate,
                    qc,
                    stats,
                    pair_fp,
                    &m1,
                    &m2,
                )?;
            }
        } else {
            handle_pe_pair_emit(
                cfg,
                &mut pe_out,
                &mut failed_w,
                &mut unpaired1_w,
                &mut unpaired2_w,
                &mut dedup_gate,
                qc,
                stats,
                pair_fp,
                &m1,
                &m2,
            )?;
        }
    }

    pe_out.finish()?;
    if let Some(mut m) = merged_out {
        m.finish()?;
    }
    flush_opt(&mut failed_w)?;
    flush_opt(&mut unpaired1_w)?;
    flush_opt(&mut unpaired2_w)?;
    Ok(())
}

fn flush_opt(w: &mut Option<FastqWriter>) -> Result<()> {
    if let Some(ref mut x) = w {
        x.flush()?;
    }
    Ok(())
}

fn write_failed_pair(
    failed_w: &mut Option<FastqWriter>,
    m1: &MutableRead,
    m2: &MutableRead,
) -> Result<()> {
    if let Some(ref mut w) = failed_w {
        m1.to_owned().write_to(&mut *w)?;
        m2.to_owned().write_to(&mut *w)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn handle_pe_pair_emit(
    cfg: &RunConfig,
    pe_out: &mut PeOut,
    failed_w: &mut Option<FastqWriter>,
    unpaired1_w: &mut Option<FastqWriter>,
    unpaired2_w: &mut Option<FastqWriter>,
    dedup_gate: &mut Option<DedupGate>,
    qc: &mut QcCollector,
    stats: &mut RunStats,
    pair_fp: u64,
    m1: &MutableRead,
    m2: &MutableRead,
) -> Result<()> {
    let q1 = read_passes_output_filters(m1, cfg);
    let q2 = read_passes_output_filters(m2, cfg);
    match (q1, q2) {
        (true, true) => {
            if let Some(ref mut g) = dedup_gate {
                if !g.allow_emit(pair_fp) {
                    return Ok(());
                }
            }
            qc.observe_after_pair(m1, m2);
            pe_out.write_pair(&m1.to_owned(), &m2.to_owned())?;
            stats.pairs_out += 1;
            stats.reads_out += 2;
        }
        (true, false) => {
            if let Some(ref mut w) = unpaired1_w {
                m1.to_owned().write_to(w)?;
                stats.reads_out += 1;
            }
            if let Some(ref mut w) = failed_w {
                m2.to_owned().write_to(w)?;
            }
        }
        (false, true) => {
            if let Some(ref mut w) = unpaired2_w {
                m2.to_owned().write_to(w)?;
                stats.reads_out += 1;
            }
            if let Some(ref mut w) = failed_w {
                m1.to_owned().write_to(w)?;
            }
        }
        (false, false) => {
            write_failed_pair(failed_w, m1, m2)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RunConfig;
    use crate::merge::reverse_complement;
    use tempfile::tempdir;

    #[test]
    fn pe_interleaved_roundtrip() {
        let dir = tempdir().unwrap();
        let inp = dir.path().join("il.fq");
        let o1 = dir.path().join("R1.fq");
        let o2 = dir.path().join("R2.fq");
        std::fs::write(&inp, "@x 1\nAAAA\n+\nIIII\n@x 2\nTTTT\n+\nIIII\n").unwrap();
        let cfg = RunConfig {
            read1: inp.clone(),
            out1: o1.clone(),
            out2: Some(o2.clone()),
            interleaved: true,
            ..Default::default()
        };
        run(&cfg).unwrap();
        let r1 = std::fs::read_to_string(&o1).unwrap();
        let r2 = std::fs::read_to_string(&o2).unwrap();
        assert!(r1.contains("AAAA"));
        assert!(r2.contains("TTTT"));
    }

    #[test]
    fn merge_constructed_overlap_writes_merged() {
        let dir = tempdir().unwrap();
        let r1p = dir.path().join("r1.fq");
        let r2p = dir.path().join("r2.fq");
        let o1 = dir.path().join("o1.fq");
        let o2 = dir.path().join("o2.fq");
        let om = dir.path().join("m.fq");
        let overlap = b"ACGTACGT";
        let mut r1s = b"AAAACCCC".to_vec();
        r1s.extend_from_slice(overlap);
        let tail = b"GGGGTTTT";
        let mut r2rc: Vec<u8> = Vec::new();
        r2rc.extend_from_slice(overlap);
        r2rc.extend_from_slice(tail);
        let r2_seq = reverse_complement(&r2rc);
        let n = r2_seq.len();
        let q = vec![b'I'; n];
        let q1 = vec![b'I'; r1s.len()];
        std::fs::write(
            &r1p,
            format!(
                "@p\n{}\n+\n{}\n",
                String::from_utf8_lossy(&r1s),
                String::from_utf8_lossy(&q1)
            ),
        )
        .unwrap();
        std::fs::write(
            &r2p,
            format!(
                "@p\n{}\n+\n{}\n",
                String::from_utf8_lossy(&r2_seq),
                String::from_utf8_lossy(&q)
            ),
        )
        .unwrap();
        let cfg = RunConfig {
            read1: r1p,
            read2: Some(r2p),
            out1: o1,
            out2: Some(o2),
            merge_pe: true,
            merged_out: Some(om.clone()),
            overlap_len_min: 8,
            overlap_match_fraction: 0.9,
            ..Default::default()
        };
        let st = run(&cfg).unwrap();
        assert!(st.merged_pairs >= 1);
        let merged = std::fs::read_to_string(&om).unwrap();
        assert!(merged.starts_with("@p"));
    }

    #[test]
    fn merge_include_unmerged_writes_pair_too() {
        let dir = tempdir().unwrap();
        let r1p = dir.path().join("r1.fq");
        let r2p = dir.path().join("r2.fq");
        let o1 = dir.path().join("o1.fq");
        let o2 = dir.path().join("o2.fq");
        let om = dir.path().join("m.fq");
        let overlap = b"ACGTACGT";
        let mut r1s = b"AAAACCCC".to_vec();
        r1s.extend_from_slice(overlap);
        let tail = b"GGGGTTTT";
        let mut r2rc: Vec<u8> = Vec::new();
        r2rc.extend_from_slice(overlap);
        r2rc.extend_from_slice(tail);
        let r2_seq = reverse_complement(&r2rc);
        let n = r2_seq.len();
        let q = vec![b'I'; n];
        let q1 = vec![b'I'; r1s.len()];
        std::fs::write(
            &r1p,
            format!(
                "@p\n{}\n+\n{}\n",
                String::from_utf8_lossy(&r1s),
                String::from_utf8_lossy(&q1)
            ),
        )
        .unwrap();
        std::fs::write(
            &r2p,
            format!(
                "@p\n{}\n+\n{}\n",
                String::from_utf8_lossy(&r2_seq),
                String::from_utf8_lossy(&q)
            ),
        )
        .unwrap();
        let cfg = RunConfig {
            read1: r1p,
            read2: Some(r2p),
            out1: o1.clone(),
            out2: Some(o2.clone()),
            merge_pe: true,
            merge_include_unmerged: true,
            merged_out: Some(om.clone()),
            overlap_len_min: 8,
            overlap_match_fraction: 0.9,
            ..Default::default()
        };
        let st = run(&cfg).unwrap();
        assert!(st.merged_pairs >= 1);
        assert!(st.pairs_out >= 1);
        assert!(std::fs::read_to_string(&o1).unwrap().contains("@p"));
        assert!(std::fs::read_to_string(&o2).unwrap().contains("@p"));
    }

    #[test]
    fn dedup_drops_second_identical_read() {
        let dir = tempdir().unwrap();
        let inp = dir.path().join("in.fq");
        let out = dir.path().join("out.fq");
        std::fs::write(
            &inp,
            "@a\nACGTACGTACGTACGTACGTACGTACGTACGT\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n@b\nACGTACGTACGTACGTACGTACGTACGTACGT\n+\nFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF\n",
        )
        .unwrap();
        let cfg = RunConfig {
            read1: inp,
            out1: out,
            dedup: true,
            ..Default::default()
        };
        let st = run(&cfg).unwrap();
        assert_eq!(st.reads_in, 2);
        assert_eq!(st.reads_out, 1);
    }

    #[test]
    fn low_complexity_filter_drops_homopolymer() {
        let dir = tempdir().unwrap();
        let inp = dir.path().join("in.fq");
        let out = dir.path().join("out.fq");
        std::fs::write(&inp, "@r\nAAAAAAAAAAAAAAAAAAAA\n+\nFFFFFFFFFFFFFFFFFFFF\n").unwrap();
        let cfg = RunConfig {
            read1: inp,
            out1: out,
            low_complexity_filter: true,
            complexity_threshold: 0.30,
            ..Default::default()
        };
        let st = run(&cfg).unwrap();
        assert_eq!(st.reads_in, 1);
        assert_eq!(st.reads_out, 0);
    }

    #[test]
    fn quality_filter_drops_low_qual_read() {
        let dir = tempdir().unwrap();
        let inp = dir.path().join("in.fq");
        let out = dir.path().join("out.fq");
        std::fs::write(&inp, "@r\nACGTACGTACGTACGT\n+\n!!!!!!!!!!!!!!!!\n").unwrap();
        let cfg = RunConfig {
            read1: inp,
            out1: out,
            disable_quality_filtering: false,
            unqualified_percent_limit: 40,
            ..Default::default()
        };
        let st = run(&cfg).unwrap();
        assert_eq!(st.reads_in, 1);
        assert_eq!(st.reads_out, 0);
    }

    #[test]
    fn failed_out_collects_filtered_se() {
        let dir = tempdir().unwrap();
        let inp = dir.path().join("in.fq");
        let out = dir.path().join("out.fq");
        let bad = dir.path().join("bad.fq");
        std::fs::write(&inp, "@r\nACGTACGTACGTACGT\n+\n!!!!!!!!!!!!!!!!\n").unwrap();
        let cfg = RunConfig {
            read1: inp,
            out1: out,
            failed_out: Some(bad.clone()),
            disable_quality_filtering: false,
            ..Default::default()
        };
        run(&cfg).unwrap();
        let failed = std::fs::read_to_string(&bad).unwrap();
        assert!(failed.contains("@r"));
    }
}
