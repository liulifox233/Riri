use plist::Value;

fn main() {
    let mut plist: Value = plist::from_file("../target/release/bundle/osx/Riri.app/Contents/Info.plist").unwrap();
    if let Value::Dictionary(ref mut dict) = plist {
        dict.insert("LSUIElement".to_string(), Value::Boolean(true));
    }
    plist.to_file_xml("../target/release/bundle/osx/Riri.app/Contents/Info.plist").unwrap();
}
