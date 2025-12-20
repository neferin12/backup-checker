mod banner;
use clap::Parser;
use owo_colors::OwoColorize;
use backup_checker::ChecksumGenerator;
use crate::banner::print_banner;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    old_folder: String,

    #[arg(short, long)]
    new_folder: String,

    #[arg(short = 'd', long, default_value = "1000")]
    max_depth: i16,

    #[clap(short, long, default_value_t, value_enum)]
    generator: ChecksumGenerator,
}


fn main() {
    let args = Args::parse();

    print_banner();
    println!("Using {} for checksums", backup_checker::get_generator_name(&args.generator).cyan().bold());
    println!("Old folder: {}", args.old_folder.cyan().bold());
    println!("New folder: {}", args.new_folder.cyan().bold());
    println!("\n");

    let missing_files = backup_checker::check_files(&args.old_folder, &args.new_folder, args.max_depth, &args.generator, true);

    println!("Missing files: {:#?}", missing_files);
}

