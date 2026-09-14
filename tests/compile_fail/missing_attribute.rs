use enum_strings::EnumStrings;

#[derive(EnumStrings)]
enum Route {
    #[strings(path = "/", label = "Home")]
    Home,
    Docs,
}

fn main() {}
