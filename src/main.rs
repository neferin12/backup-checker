mod banner;
use crate::banner::print_banner;
use backup_checker::ChecksumGenerator;
use clap::Parser;
use owo_colors::OwoColorize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    old_folder: Option<String>,

    #[arg(short, long)]
    new_folder: Option<String>,

    #[arg(short = 'f', long)]
    duplicates_folder: Option<String>,

    #[arg(short = 'd', long, default_value = "1000")]
    max_depth: i16,

    #[clap(short, long, default_value_t, value_enum)]
    generator: ChecksumGenerator,
}

fn main() {
    let args = Args::parse();

    print_banner();
    println!(
        "Using {} for checksums",
        backup_checker::get_generator_name(&args.generator)
            .cyan()
            .bold()
    );

    if let Some(folder) = args.duplicates_folder {
        println!("Duplicates folder: {}", folder.cyan().bold());
        println!();

        let duplicate_files =
            backup_checker::find_duplicate_files(&folder, args.max_depth, &args.generator, true);

        println!("Duplicate files: {:#?}", duplicate_files);
    } else if let (Some(old_folder), Some(new_folder)) = (args.old_folder, args.new_folder) {
        println!("Old folder: {}", old_folder.cyan().bold());
        println!("New folder: {}", new_folder.cyan().bold());
        println!();

        let missing_files = backup_checker::check_files(
            &old_folder,
            &new_folder,
            args.max_depth,
            &args.generator,
            true,
        );

        println!("Missing files: {:#?}", missing_files);
    } else {
        eprintln!(
            "Either --duplicates-folder or both --old-folder and --new-folder must be provided."
        );
        std::process::exit(2);
    }
}
