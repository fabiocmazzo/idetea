///|
/// Adds the numbers from 1 to `n`.
pub fn sum_to(n : Int32) -> Int32 {
  let mut total = 0
  for i = 1; i <= n; i = i + 1 {
    total = total + i
  }
  total
}

///|
/// Greets someone, using string interpolation.
pub fn greet(name : String) -> String {
  "Hello, \{name}! 1 + ... + 10 = \{sum_to(10)}"
}
