use std::fs::File;
use std::io::{Write, BufReader, BufRead};

struct Book {
    title: String,
    author: String,
    year: u16,
}

fn save_books(books: &Vec<Book>, filename: &str) {
    // TODO: Implement this function
    // Hint: Use File::create() and write!() macro
    let mut file = File::create(filename).expect("Failed to create file");
    for book in books {
        writeln!(file, "{}", book.title).expect("Failed to write title");
        writeln!(file, "{}", book.author).expect("Failed to write author");
        writeln!(file, "{}", book.year).expect("Failed to write year");
    }

}

fn load_books(filename: &str) -> Vec<Book> {
    // TODO: Implement this function
    // Hint: Use File::open() and BufReader
    // Use File::open and BufReader to read line by line and parse
    let file = File::open(filename).expect("Failed to open file");
    let reader = BufReader::new(file);
    let mut books = Vec::new();
    let mut lines = reader.lines();

    while let Some(title_line) = lines.next() {
        let title = title_line.unwrap().trim().to_string();
        let author = lines.next().unwrap().unwrap().trim().to_string();
        let year_str = lines.next().unwrap().unwrap().trim().to_string();
        
        if let Ok(year) = year_str.parse::<u16>() {
            books.push(Book { title, author, year });
        }
    }
books
}

fn main() {
    let books = vec![
        Book { title: "1984".to_string(), author: "George Orwell".to_string(), year: 1949 },
        Book { title: "To Kill a Mockingbird".to_string(), author: "Harper Lee".to_string(), year: 1960 },
    ];

    save_books(&books, "books.txt");
    println!("Books saved to file.");

    let loaded_books = load_books("books.txt");
    println!("Loaded books:");
    for book in loaded_books {
        println!("{} by {}, published in {}", book.title, book.author, book.year);
    }
}