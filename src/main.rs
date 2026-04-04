extern crate chrono;

use chrono::Local;

fn make_url(leverancier: i32) -> String {
    let date = Local::now();
    let date_string = date.format("%Y-%m-%d");
    format!(
        "https://www.stroomperuur.nl/ajax/tarieven.php?leverancier={}&datum={}",
        leverancier, date_string
    )
}

fn main() {
    let url = make_url(2);
    println!("{}", url);
}
