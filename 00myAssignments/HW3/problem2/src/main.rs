fn clone_and_modify(s: &String) -> String {
    // Your code here
    let mut result = s.clone();
    let s2 = ", World!";
    result.push_str(s2);
    result

}

fn main() {
    let s = String::from("Hello, ");
    let modified = clone_and_modify(&s);
    println!("Original: {}", s); // Should print: "Original: Hello, "
    println!("Modified: {}", modified); // Should print: "Modified: Hello, World!"
}