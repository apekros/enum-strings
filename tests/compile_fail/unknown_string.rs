use enum_strings::EnumStrings;

#[derive(EnumStrings)]
enum Route {
    #[strings(path = "/", label = "Home")]
    Home,
    #[strings(path = "/docs", lable = "Docs")]
    Docs,
}

fn main() {}
