// Adds the numbers from 1 to n.
export fn upto(n: int) int = {
	let total = 0;
	for (let i = 1; i <= n; i += 1) {
		total += i;
	};
	return total;
};

@test fn upto() void = {
	assert(upto(10) == 55);
	assert(upto(0) == 0);
};
