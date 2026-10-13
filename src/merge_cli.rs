use std::env;
use stichup::stitch::merge_pdfs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: merge_cli a.pdf b.pdf out.pdf");
        return;
    }
    let paths = vec![args[1].clone(), args[2].clone()];
    match merge_pdfs(&paths) {
        Ok(mut d) => {
            let _ = d.save(&args[3]);
            println!("saved {}", args[3]);
        }
        Err(e) => eprintln!("err {}", e),
    }
}
