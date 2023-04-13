const USER_AGENT: &str = concat!(
    "sqlite-package-manager/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/asg017/sqlite-package-manager)"
);

pub fn http_get(url: &str) -> ureq::Request {
    ureq::get(url).set("User-Agent", USER_AGENT)
}
