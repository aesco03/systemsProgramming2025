//#[allow(unused_variables, unused_mut)]

fn sum(total: &mut i32, low: i32, high: i32) {
    // Write your code here!
    let mut total = 0;

    for i in low..=high{
        *total += i;
    }
}

fn sum_recursive(total: &mut i32, low: i32, high: i32) {
    // Write your code here!
    // Sum formula: n(n+1)/2
    if low > high {
        return;
    }
    *total += low;
    sum_recursive(total, low + 1, high);
}

fn main() {
    // create necessary variables and test your function for low 0 high 100
    // total should be 5050
    let mut total = 0;
    let low = 0;
    let high = 100;
    sum(&mut total, low, high);  // pass by mutable reference
    println!("Total sum: {}", total);
}