use enum_strings::EnumStrings;

#[derive(EnumStrings)]
enum Route {
    #[strings(path = "/", label = "Home")]
    Home,
    #[strings(path = "/docs")]
    Docs,
}

fn main() {}
