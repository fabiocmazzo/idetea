use fmt;
use sum;

export fn main() void = {
	const n = 10;
	fmt::printfln("1 + ... + {} = {}", n, sum::upto(n))!;
};
