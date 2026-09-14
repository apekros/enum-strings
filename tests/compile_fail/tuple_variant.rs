use enum_strings::EnumStrings;

#[derive(EnumStrings)]
enum Route {
    #[strings(path = "/")]
    Home(u8),
}

fn main() {}
