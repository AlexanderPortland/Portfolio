use alohomora::sandbox::AlohomoraSandbox;

extern crate alohomora;
extern crate chrono;

pub const NAIVE_DATE_FMT: &str = "%Y-%m-%d";

#[AlohomoraSandbox]
fn naive_date_str((date, format): (chrono::NaiveDate, bool)) -> String {
    match format {
        true => date.to_string(),
        false => date.format(NAIVE_DATE_FMT).to_string(),
    }
}

// #[AlohomoraSandbox]
// fn serde_to_sandbox<T: Serialize>(t: T) -> String {
//     serde_json::to_string(&t).unwrap()
// }