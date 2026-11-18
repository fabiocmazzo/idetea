/// A tea order: which tea and how many cups.
struct Order {
    tea: String,
    cups: u32,
}

fn price(cups: u32, unit: f64) -> f64 {
    cups as f64 * unit
}

fn stock() -> HashMap<String, u32> {
    let mut shelf = HashMap::new();
    shelf.insert("Sencha".to_string(), 24);
    shelf
}

fn main() {
    let order = Order { tea: "Sencha".into(), cups: 3 };
    let total = price(order.cups, 4.5);
    let left = stock()["Sencha"] - order.cups;
    println!("{} cups of {}: ${total:.2}, {left} left", order.cups, order.tea);
}
