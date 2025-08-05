/*
Create a Rust program that converts temperatures between Fahrenheit and Celsius. The program should:

1. Declare a constant for the freezing point of water in Fahrenheit (32°F).

2. Implement two functions:
fahrenheit_to_celsius(f: f64) -> f64: Converts Fahrenheit to Celsius
celsius_to_fahrenheit(c: f64) -> f64: Converts Celsius to Fahrenheit

3. In the main function:
Declare a mutable variable with a temperature in Fahrenheit
Convert it to Celsius using your function and print the result
Use a loop to convert and print the next 5 integer temperatures (e.g., if you start with 32°F, print conversions for 33°F, 34°F, 35°F, 36°F, and 37°F)

*/
    
const F_FREEZE_POINT: i64 = 32;

fn fahrenheit_to_celsius(f: f64) -> f64{
    (f - F_FREEZE_POINT as f64) * 5.0 / 9.0
    }

fn celsius_to_fahrenheit(c: f64) -> f64{
    c * 9.0 / 5.0 + F_FREEZE_POINT as f64
    }

//______Assignment 2: Number Analyzer____________

fn is_even(n: i32) -> bool{
    n % 2 == 0
}

// _________ Assignment 3: Guess Game___________
fn check_guess(guess: i32, secret: i32) -> i32{
    if guess == secret {
        0
    } else if guess > secret {
        1
    } else {
        -1
    }
}

fn main() {

let mut temp_f = 32f64;
println!("Fahrenheit before conversion:{}",temp_f);

let mut temp_c = fahrenheit_to_celsius(temp_f);

println!("New Fahrenheit temp in Celsius: {}", temp_c);

for _ in 0..5 {
    temp_f += 1.0;

    temp_c = fahrenheit_to_celsius(temp_f);
    println!("New {} F = {} C", temp_f, temp_c)

}
//___Assignment 2___
let arr = [4, 2, 7, 1, 5, 9, 7, 12, 3, 10];

let max = arr.len();

for n in 0..max{
    if is_even(n as i32){
        println!("True: {} is even", n);
    }else {
        println!("False: {} is odd", n);
    }

    if n % 3 == 0 && n % 5 == 0{
            println!("FizzBuzz");
    }else if n % 3 == 0 {
            println!("Fizz");
    } else if n % 5 == 0 {
            println!("Buzz");
        } 
}
let mut idx = 0;
let mut sum = 0;

while idx < max {
    sum += arr[idx];
    idx+=1;
}
println!("Sum of all numbers in arr: {}", sum);

let mut max_num = arr[0];
for n in arr{
    if n > max_num{
        max_num = n;
    }
}
println!("The largest element in arr is: {}", max_num);

// _______ Assignment 3________
let secret = 77i32;

let mut guesses = 0;
 loop {
        let mut guess = 77; //(simulating user input)
        
        // Change guess based on attempt number
        if guesses == 1 {
            guess = 5;
        } else if guesses == 2 {
            guess = 7;
        }
        
        guesses += 1;
        let result = check_guess(guess, secret);
        
        if result == 0 {
            println!("Correct! The number was {}", guess);
            break;
        } else if result == 1 {
            println!("Your guess {} is too high", guess);
        } else {
            println!("Your guess {} is too low", guess);
        }
    }
    
    println!("It took {} guesses", guesses);


}


