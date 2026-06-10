//! Runtime configuration (CLI + library).

use std::path::PathBuf;

/// Where to read UMI bases from before they are moved into the read name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UmiSource {
    #[default]
    None,
    Read1Prefix(usize),
    Read2Prefix(usize),
}

/// Full preprocessing configuration.
#[derive(Debug, Clone)]
pub struct RunConfig {
    pub read1: PathBuf,
    pub read2: Option<PathBuf>,
    pub out1: PathBuf,
    pub out2: Option<PathBuf>,
    pub merged_out: Option<PathBuf>,
    pub interleaved: bool,

    pub qual_threshold: u8,
    pub unqualified_len_limit: usize,
    pub cut_window_size: usize,
    pub cut_mean_qual: u8,
    pub cut_front_window_size: usize,
    pub cut_front_mean_qual: u8,
    pub trim_tail_qual: bool,

    pub cut_right_window_size: usize,
    pub cut_right_mean_qual: u8,

    pub adapter_r1: Option<Vec<u8>>,
    pub adapter_r2: Option<Vec<u8>>,
    pub adapter_min_match: usize,
    pub disable_adapter_trimming: bool,

    pub trim_poly_g: bool,
    pub poly_g_min_len: usize,
    pub trim_poly_x: bool,
    pub poly_x_base: u8,
    pub poly_x_min_len: usize,

    pub trim_front1: usize,
    pub trim_tail1: usize,
    /// 0 = no max length cap.
    pub max_len1: usize,
    pub trim_front2: Option<usize>,
    pub trim_tail2: Option<usize>,
    pub max_len2: Option<usize>,

    pub length_required: Option<usize>,
    pub length_limit: Option<usize>,
    pub disable_length_filtering: bool,

    pub disable_quality_filtering: bool,
    pub unqualified_percent_limit: u8,
    pub n_base_limit: usize,
    pub average_qual_required: u8,

    pub merge_pe: bool,
    pub overlap_len_min: usize,
    pub overlap_match_fraction: f64,
    pub overlap_diff_max: usize,
    pub overlap_diff_percent_limit: f64,
    pub pe_overlap_correction: bool,

    pub umi_source: UmiSource,
    pub umi_skip_read_name_prefix: bool,
    pub umi_prefix: Option<String>,
    pub umi_skip: usize,
    pub umi_delim: String,

    pub split_reads_per_file: Option<u64>,
    pub split_prefix_digits: Option<u8>,

    pub json_report: Option<PathBuf>,
    pub html_report: Option<PathBuf>,
    pub report_title: Option<String>,

    pub phred64: bool,
    pub stdin_gzip: bool,
    pub stdout_gzip: bool,
    pub gzip_compression_level: u32,

    pub reads_to_process: Option<u64>,
    pub failed_out: Option<PathBuf>,
    pub unpaired1: Option<PathBuf>,
    pub unpaired2: Option<PathBuf>,

    /// `--dedup` / `-D`: drop reads (or pairs) whose pre-trim fingerprint was already emitted.
    pub dedup: bool,
    /// `--low_complexity_filter` / `-y`
    pub low_complexity_filter: bool,
    /// `--complexity_threshold` as fraction 0.0–1.0 (fastp uses 0–100; default 30 → 0.30).
    pub complexity_threshold: f64,
    /// `--include_unmerged` with `--merge`: after a successful merge, also write trimmed pairs to R1/R2 outputs.
    pub merge_include_unmerged: bool,

    pub dont_eval_duplication: bool,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            read1: PathBuf::new(),
            read2: None,
            out1: PathBuf::new(),
            out2: None,
            merged_out: None,
            interleaved: false,
            qual_threshold: 15,
            unqualified_len_limit: 10,
            cut_window_size: 4,
            cut_mean_qual: 20,
            cut_front_window_size: 4,
            cut_front_mean_qual: 20,
            trim_tail_qual: true,
            cut_right_window_size: 0,
            cut_right_mean_qual: 20,
            adapter_r1: None,
            adapter_r2: None,
            adapter_min_match: 8,
            disable_adapter_trimming: false,
            trim_poly_g: false,
            poly_g_min_len: 10,
            trim_poly_x: false,
            poly_x_base: b'A',
            poly_x_min_len: 10,
            trim_front1: 0,
            trim_tail1: 0,
            max_len1: 0,
            trim_front2: None,
            trim_tail2: None,
            max_len2: None,
            length_required: None,
            length_limit: None,
            disable_length_filtering: false,
            disable_quality_filtering: true,
            unqualified_percent_limit: 40,
            n_base_limit: 5,
            average_qual_required: 0,
            merge_pe: false,
            overlap_len_min: 30,
            overlap_match_fraction: 0.85,
            overlap_diff_max: 5,
            overlap_diff_percent_limit: 20.0,
            pe_overlap_correction: false,
            umi_source: UmiSource::None,
            umi_skip_read_name_prefix: false,
            umi_prefix: None,
            umi_skip: 0,
            umi_delim: ":".into(),
            split_reads_per_file: None,
            split_prefix_digits: None,
            json_report: None,
            html_report: None,
            report_title: None,
            phred64: false,
            stdin_gzip: false,
            stdout_gzip: false,
            gzip_compression_level: 4,
            reads_to_process: None,
            failed_out: None,
            unpaired1: None,
            unpaired2: None,
            dedup: false,
            low_complexity_filter: false,
            complexity_threshold: 0.30,
            merge_include_unmerged: false,
            dont_eval_duplication: false,
        }
    }
}

impl RunConfig {
    pub fn is_paired(&self) -> bool {
        self.read2.is_some() || self.interleaved
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.is_paired()
            && self.out2.is_none()
            && !self.merge_pe
            && !crate::io::path_is_stdio_dash(&self.out1)
        {
            return Err(
                "paired-end mode requires -O/--out2 unless --merge or stdout (-o -)".into(),
            );
        }
        if self.interleaved && self.read2.is_some() {
            return Err("use either --interleaved or -I/--in2, not both".into());
        }
        if self.merge_pe && !self.is_paired() {
            return Err("--merge requires paired input (-I or --interleaved)".into());
        }
        if self.merge_pe && self.merged_out.is_none() {
            return Err("--merge requires --merged-out".into());
        }
        if self.split_reads_per_file == Some(0) {
            return Err("--split must be > 0".into());
        }
        if self.merge_include_unmerged && !self.merge_pe {
            return Err("--include_unmerged requires --merge".into());
        }
        if self.split_reads_per_file.is_some()
            && (crate::io::path_is_stdio_dash(&self.out1)
                || self
                    .out2
                    .as_ref()
                    .is_some_and(|p| crate::io::path_is_stdio_dash(p.as_path())))
        {
            return Err("--split cannot be used with stdout ('-') outputs".into());
        }
        if self.merge_pe
            && (crate::io::path_is_stdio_dash(&self.out1)
                || self
                    .merged_out
                    .as_ref()
                    .is_some_and(|p| crate::io::path_is_stdio_dash(p.as_path())))
        {
            return Err("merge mode with stdout is not supported; use file paths for --merged-out and -o/-O".into());
        }
        if !(0.0..=1.0).contains(&self.complexity_threshold) {
            return Err("complexity threshold must be between 0.0 and 1.0".into());
        }
        if self.unqualified_percent_limit > 100 {
            return Err("unqualified_percent_limit must be 0–100".into());
        }
        if !(1..=9).contains(&self.gzip_compression_level) {
            return Err("gzip compression level must be 1–9".into());
        }
        if self.stdout_gzip && self.split_reads_per_file.is_some() {
            return Err("--stdout-gzip cannot be used with --split".into());
        }
        if (self.unpaired1.is_some() || self.unpaired2.is_some()) && !self.is_paired() {
            return Err("--unpaired1/--unpaired2 require paired-end input".into());
        }
        if self.split_prefix_digits.is_some_and(|d| d > 10) {
            return Err("split_prefix_digits must be <= 10".into());
        }
        Ok(())
    }
}
