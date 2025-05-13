use caesura::ArgumentsParser;

fn main() {
    let markdown: String = clap_markdown::help_markdown::<ArgumentsParser>();
    println!("{markdown}");
}
