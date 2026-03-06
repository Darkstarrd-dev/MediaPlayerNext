use mediaplayernext_lib::runtime_smoke_check_entry;

fn main() {
    if let Err(error) = runtime_smoke_check_entry() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
