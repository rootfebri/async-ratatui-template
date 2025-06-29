use crate::header_static;

header_static! {
  pub SECTRAIL_HEADER = [
    "host": "securitytrails.com",
    "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:139.0) Gecko/20100101 Firefox/139.0",
    "accept": "*/*",
    "accept-language": "en-US,en;q=0.5",
    "accept-encoding": "gzip, deflate, br, zstd",
    "referer": "https://securitytrails.com/app/account",
    "x-nextjs-data": "1",
    "sec-gpc": "1",
    "connection": "keep-alive",
    "sec-fetch-dest": "empty",
    "sec-fetch-mode": "cors",
    "sec-fetch-site": "same-origin",
    "priority": "u=0",
    "te": "trailers",
  ]
}

mod client;
pub use client::*;
pub mod jsons;
mod sec_trail_puppeteer;
pub use sec_trail_puppeteer::*;
mod scp_impls;
