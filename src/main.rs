use anyhow::Context;
use clap::Parser;
use fastp_rs::config::RunConfig;
use fastp_rs::config::UmiSource;
use fastp_rs::pipeline::run;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "fastp-rs")]
#[command(version)]
#[command(about = "FASTQ preprocessor (fastp-inspired)", long_about = None)]
struct Cli {
    #[arg(short = 'i', long = "in1", value_name = "PATH")]
    read1: Option<PathBuf>,

    #[arg(short = 'I', long = "in2", value_name = "PATH")]
    read2: Option<PathBuf>,

    #[arg(short = 'o', long = "out1", value_name = "PATH")]
    out1: Option<PathBuf>,

    #[arg(short = 'O', long = "out2", value_name = "PATH")]
    out2: Option<PathBuf>,

    /// Interleaved PE in `-i` (R1 record then R2 record, repeating).
    #[arg(long)]
    interleaved: bool,

    /// Merge overlapping PE reads; requires `--merged-out`.
    #[arg(short = 'm', long = "merge")]
    merge: bool,

    #[arg(long = "merged-out", value_name = "PATH")]
    merged_out: Option<PathBuf>,

    #[arg(long, default_value_t = 15)]
    qualified_quality_phred: u8,

    #[arg(short = 'u', long = "unqualified_percent_limit", default_value_t = 40)]
    unqualified_percent_limit: u8,

    #[arg(short = 'n', long = "n_base_limit", default_value_t = 5)]
    n_base_limit: usize,

    #[arg(short = 'e', long = "average_qual", default_value_t = 0)]
    average_qual: u8,

    #[arg(long, default_value_t = 10)]
    unqualified_base_limit: usize,

    #[arg(long, default_value_t = 4)]
    cut_window_size: usize,

    #[arg(long = "cut_mean_quality", default_value_t = 20)]
    cut_mean_quality: u8,

    #[arg(long, default_value_t = 4)]
    cut_front_window_size: usize,

    #[arg(long = "cut_front_mean_quality", default_value_t = 20)]
    cut_front_mean_quality: u8,

    /// Aggressive 5′→3′ quality cut (same window/mean as `--cut_window_size` / `--cut_mean_quality` unless overridden below).
    #[arg(short = 'r', long = "cut_right")]
    cut_right: bool,

    #[arg(long = "cut_right_window_size")]
    cut_right_window_size: Option<usize>,

    #[arg(long = "cut_right_mean_quality")]
    cut_right_mean_quality: Option<u8>,

    #[arg(long, default_value_t = true)]
    trim_tail: bool,

    #[arg(long = "adapter_sequence")]
    adapter_r1: Option<String>,

    #[arg(long = "adapter_sequence_r2")]
    adapter_r2: Option<String>,

    #[arg(long, default_value_t = 8)]
    adapter_min_match: usize,

    #[arg(short = 'A', long = "disable_adapter_trimming")]
    disable_adapter_trimming: bool,

    #[arg(long)]
    trim_poly_g: bool,

    #[arg(long, default_value_t = 10)]
    poly_g_min_len: usize,

    #[arg(long)]
    trim_poly_x: bool,

    #[arg(long, default_value_t = 'A')]
    poly_x_base: char,

    #[arg(long, default_value_t = 10)]
    poly_x_min_len: usize,

    #[arg(short = 'f', long = "trim_front1", default_value_t = 0)]
    trim_front1: usize,

    #[arg(short = 't', long = "trim_tail1", default_value_t = 0)]
    trim_tail1: usize,

    #[arg(short = 'b', long = "max_len1", default_value_t = 0)]
    max_len1: usize,

    #[arg(short = 'F', long = "trim_front2")]
    trim_front2: Option<usize>,

    #[arg(short = 'T', long = "trim_tail2")]
    trim_tail2: Option<usize>,

    #[arg(short = 'B', long = "max_len2")]
    max_len2: Option<usize>,

    #[arg(long = "length_required")]
    length_required: Option<usize>,

    #[arg(long = "length_limit")]
    length_limit: Option<usize>,

    #[arg(short = 'L', long = "disable_length_filtering")]
    disable_length_filtering: bool,

    #[arg(short = 'Q', long = "disable_quality_filtering")]
    disable_quality_filtering: bool,

    #[arg(long, default_value_t = 30)]
    overlap_len_min: usize,

    #[arg(long, default_value_t = 0.85)]
    overlap_match_fraction: f64,

    #[arg(long = "overlap_diff_limit", default_value_t = 5)]
    overlap_diff_limit: usize,

    #[arg(long = "overlap_diff_percent_limit", default_value_t = 20.0)]
    overlap_diff_percent_limit: f64,

    #[arg(short = 'c', long = "correction")]
    correction: bool,

    /// UMI length from start of R1 (`read1`) or R2 (`read2`).
    #[arg(long)]
    umi_len: Option<usize>,

    #[arg(long, value_enum, default_value = "none")]
    umi_loc: UmiLocCli,

    #[arg(long = "umi_prefix")]
    umi_prefix: Option<String>,

    #[arg(long = "umi_skip", default_value_t = 0)]
    umi_skip: usize,

    #[arg(long = "umi_delim", default_value = ":")]
    umi_delim: String,

    /// Split output every N reads (separate part files).
    #[arg(long = "split", value_name = "N")]
    split_reads: Option<u64>,

    #[arg(short = 'd', long = "split_prefix_digits")]
    split_prefix_digits: Option<u8>,

    #[arg(long = "json", value_name = "PATH")]
    json_report: Option<PathBuf>,

    #[arg(long = "html", value_name = "PATH")]
    html_report: Option<PathBuf>,

    #[arg(short = 'R', long = "report_title")]
    report_title: Option<String>,

    /// Read R1 from stdin (plain FASTQ); same as `-i -`.
    #[arg(long)]
    stdin: bool,

    /// Decompress gzip from stdin (only with stdin / `-i -`).
    #[arg(long = "stdin-gzip")]
    stdin_gzip: bool,

    /// Write passing reads to stdout (plain or gzip). PE: interleaved R1 then R2 per pair (`-o -`, omit `-O`).
    #[arg(long)]
    stdout: bool,

    /// gzip-compress stdout (only with `--stdout` / `-o -`).
    #[arg(long = "stdout-gzip")]
    stdout_gzip: bool,

    /// gzip compression level for `.gz` outputs and `--stdout-gzip` (1–9).
    #[arg(short = 'z', long = "compression", default_value_t = 4)]
    compression: u32,

    /// Input uses Phred+64 qualities (converted to Phred+33 internally).
    #[arg(short = '6', long = "phred64")]
    phred64: bool,

    /// Stop after this many reads (SE) or read pairs (PE); 0 = all.
    #[arg(long = "reads_to_process", default_value_t = 0)]
    reads_to_process: u64,

    #[arg(long = "failed_out", value_name = "PATH")]
    failed_out: Option<PathBuf>,

    #[arg(long = "unpaired1", value_name = "PATH")]
    unpaired1: Option<PathBuf>,

    #[arg(long = "unpaired2", value_name = "PATH")]
    unpaired2: Option<PathBuf>,

    /// Drop duplicate reads / pairs (same 48 bp + length fingerprint as first seen).
    #[arg(long = "dedup", short = 'D')]
    dedup: bool,

    #[arg(long = "dont_eval_duplication")]
    dont_eval_duplication: bool,

    /// Filter out low-complexity reads (fastp `-y`).
    #[arg(long = "low_complexity_filter", short = 'y')]
    low_complexity_filter: bool,

    /// Minimum adjacent-difference fraction 0–100 (fastp `-Y`, default 30).
    #[arg(long = "complexity_threshold", short = 'Y', default_value_t = 30)]
    complexity_threshold_pct: u8,

    /// With `--merge`, also write unmerged trimmed pairs to `-o`/`-O`.
    #[arg(long = "include_unmerged")]
    include_unmerged: bool,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
enum UmiLocCli {
    #[default]
    None,
    Read1,
    Read2,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let read1 = if cli.stdin {
        cli.read1
            .clone()
            .unwrap_or_else(|| std::path::PathBuf::from("-"))
    } else {
        let Some(r) = cli.read1.clone() else {
            eprintln!("fastp-rs: specify -i/--in1 or --stdin.");
            std::process::exit(2);
        };
        r
    };

    let out1 = if cli.stdout {
        std::path::PathBuf::from("-")
    } else {
        let Some(o) = cli.out1.clone() else {
            eprintln!("fastp-rs: specify -o/--out1 or --stdout.");
            std::process::exit(2);
        };
        o
    };

    let out2 = if cli.stdout { None } else { cli.out2.clone() };

    let complexity_threshold = (cli.complexity_threshold_pct.min(100) as f64) / 100.0;

    let umi_source = match (cli.umi_loc, cli.umi_len) {
        (UmiLocCli::Read1, Some(n)) => UmiSource::Read1Prefix(n),
        (UmiLocCli::Read2, Some(n)) => UmiSource::Read2Prefix(n),
        _ => UmiSource::None,
    };

    let (cr_w, cr_q) = if cli.cut_right {
        (
            cli.cut_right_window_size.unwrap_or(cli.cut_window_size),
            cli.cut_right_mean_quality.unwrap_or(cli.cut_mean_quality),
        )
    } else {
        (0usize, 0u8)
    };

    let reads_to_process = if cli.reads_to_process == 0 {
        None
    } else {
        Some(cli.reads_to_process)
    };

    let cfg = RunConfig {
        read1,
        read2: cli.read2.clone(),
        out1,
        out2,
        merged_out: cli.merged_out.clone(),
        interleaved: cli.interleaved,
        qual_threshold: cli.qualified_quality_phred,
        unqualified_len_limit: cli.unqualified_base_limit,
        cut_window_size: cli.cut_window_size,
        cut_mean_qual: cli.cut_mean_quality,
        cut_front_window_size: cli.cut_front_window_size,
        cut_front_mean_qual: cli.cut_front_mean_quality,
        trim_tail_qual: cli.trim_tail,
        cut_right_window_size: cr_w,
        cut_right_mean_qual: cr_q,
        adapter_r1: cli.adapter_r1.as_ref().map(|s| s.as_bytes().to_vec()),
        adapter_r2: cli.adapter_r2.as_ref().map(|s| s.as_bytes().to_vec()),
        adapter_min_match: cli.adapter_min_match,
        disable_adapter_trimming: cli.disable_adapter_trimming,
        trim_poly_g: cli.trim_poly_g,
        poly_g_min_len: cli.poly_g_min_len,
        trim_poly_x: cli.trim_poly_x,
        poly_x_base: cli.poly_x_base.to_ascii_uppercase() as u8,
        poly_x_min_len: cli.poly_x_min_len,
        trim_front1: cli.trim_front1,
        trim_tail1: cli.trim_tail1,
        max_len1: cli.max_len1,
        trim_front2: cli.trim_front2,
        trim_tail2: cli.trim_tail2,
        max_len2: cli.max_len2,
        length_required: cli.length_required,
        length_limit: cli.length_limit,
        disable_length_filtering: cli.disable_length_filtering,
        disable_quality_filtering: cli.disable_quality_filtering,
        unqualified_percent_limit: cli.unqualified_percent_limit,
        n_base_limit: cli.n_base_limit,
        average_qual_required: cli.average_qual,
        merge_pe: cli.merge,
        overlap_len_min: cli.overlap_len_min,
        overlap_match_fraction: cli.overlap_match_fraction,
        overlap_diff_max: cli.overlap_diff_limit,
        overlap_diff_percent_limit: cli.overlap_diff_percent_limit,
        pe_overlap_correction: cli.correction,
        umi_source,
        umi_skip_read_name_prefix: false,
        umi_prefix: cli.umi_prefix.clone(),
        umi_skip: cli.umi_skip,
        umi_delim: cli.umi_delim.clone(),
        split_reads_per_file: cli.split_reads,
        split_prefix_digits: cli.split_prefix_digits,
        json_report: cli.json_report.clone(),
        html_report: cli.html_report.clone(),
        report_title: cli.report_title.clone(),
        phred64: cli.phred64,
        stdin_gzip: cli.stdin_gzip,
        stdout_gzip: cli.stdout_gzip,
        gzip_compression_level: cli.compression.clamp(1, 9),
        reads_to_process,
        failed_out: cli.failed_out.clone(),
        unpaired1: cli.unpaired1.clone(),
        unpaired2: cli.unpaired2.clone().or(cli.unpaired1.clone()),
        dedup: cli.dedup,
        low_complexity_filter: cli.low_complexity_filter,
        complexity_threshold,
        merge_include_unmerged: cli.include_unmerged,
        dont_eval_duplication: cli.dont_eval_duplication,
    };

    cfg.validate()
        .map_err(|s| anyhow::anyhow!("invalid configuration: {s}"))?;

    let stats = run(&cfg)
        .map_err(|e| anyhow::anyhow!(e))
        .context("run pipeline")?;

    tracing::info!(
        reads_in = stats.reads_in,
        reads_out = stats.reads_out,
        pairs_in = stats.pairs_in,
        pairs_out = stats.pairs_out,
        merged = stats.merged_pairs,
        "done"
    );
    Ok(())
}

fn init_tracing(verbose: u8) {
    let filter = match verbose {
        0 => tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        1 => tracing_subscriber::EnvFilter::new("info"),
        _ => tracing_subscriber::EnvFilter::new("debug"),
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}
