// The CodeDecode problem set (Polygon contest 62114), ported from each
// problem's reference solution. Every hidden function takes the raw input
// exactly as the problem formats it and returns the expected output, or an
// error message when the input breaks the problem's constraints.

use std::{collections::VecDeque, str::FromStr};

pub struct Question {
    pub statement: &'static str,
    pub solve: fn(&str) -> Result<String, String>,
}

pub const QUESTIONS: [Question; 10] = [
    Question {
        statement: "Salad with a Twist\n\
You have intercepted an encrypted string p, and a reliable source has leaked a mysterious keyword q. Your mission is to figure out the scheme and decode the message.\n\
Input format: t (1 ≤ t ≤ 100) test cases. Each test case has n and m (1 ≤ m ≤ n ≤ 10^5), then the strings p and q of lengths n and m. p, q and s use lowercase Latin letters only.\n\
Output format: the decoded string s of length n for each test case.\n\
Input: 1\n\
5 3\n\
manya abc\n\
Output: lcmxc",
        solve: string_cipher,
    },
    Question {
        statement: "Thor and The Lightning Tree\n\
Thor is studying a mystical tree in Asgard with n vertices numbered 1 to n. For each vertex u he wants to find its lightning power c[u]. Help Thor compute c_1, c_2, ..., c_n.\n\
Input format: t (1 ≤ t ≤ 10^4) test cases. Each test case has n (1 ≤ n ≤ 2·10^5), then n-1 lines u v describing the edges of a tree. The sum of n over all test cases is at most 2·10^5.\n\
Output format: for each test case, n integers c_1 ... c_n.\n\
Input: 1\n\
5\n\
1 2\n\
2 3\n\
3 4\n\
2 5\n\
Output: 3 2 2 3 3",
        solve: lightning_tree,
    },
    Question {
        statement: "Random Arrays\n\
Divij takes two arrays a and b and a binary string s of length n, then computes a third array c using some weird operations. It is known that Divij likes to randomly reverse stuff! Compute c_1, c_2, ..., c_n.\n\
Input format: t (1 ≤ t ≤ 10^4) test cases. Each test case has n (1 ≤ n ≤ 2·10^5), a line of n integers a_i, a line of n integers b_i (0 ≤ a_i, b_i ≤ 10^9), and a binary string s of length n. The sum of n is at most 2·10^5.\n\
Output format: for each test case, n integers c_1 ... c_n.\n\
Input: 1\n\
3\n\
1 2 3\n\
1 2 3\n\
010\n\
Output: 1 268435456 3",
        solve: random_arrays,
    },
    Question {
        statement: "Encrypted Arrays\n\
Monocarp takes two arrays a and b and a binary string s of length n, then computes a third array c using some operations. Compute c_1, c_2, ..., c_n.\n\
Input format: t (1 ≤ t ≤ 10^4) test cases. Each test case has n (1 ≤ n ≤ 2·10^5), a line of n integers a_i, a line of n integers b_i (0 ≤ a_i, b_i ≤ 10^9), and a binary string s of length n. The sum of n is at most 2·10^5.\n\
Output format: for each test case, n integers c_1 ... c_n.\n\
Input: 1\n\
3\n\
1 2 3\n\
4 5 6\n\
011\n\
Output: 5 6 5",
        solve: encrypted_arrays,
    },
    Question {
        statement: "Treemaxxing\n\
You are given a tree. Find a_1, a_2, ..., a_n.\n\
Input format: n (2 ≤ n ≤ 2000), then n-1 lines u v (u ≠ v) describing the edges of a tree.\n\
Output format: n integers a_1 ... a_n.\n\
Input: 4\n\
1 2\n\
2 3\n\
2 4\n\
Output: 2 1 2 2",
        solve: treemaxxing,
    },
    Question {
        statement: "Drow, I Can Ob If\n\
Big E was given a lowercase string and instructed to check if its [REDACTED]. He paused, reconsidered, and attempted to [REDACTED]. Finally, he answered with either YES or NO. Determine Big E's answer.\n\
Input format: n (3 ≤ n ≤ 20), then a lowercase string s of length n.\n\
Output format: YES or NO.\n\
Input: 3\n\
sum\n\
Output: YES",
        solve: fib_word,
    },
    Question {
        statement: "Prime Murmurs\n\
King and Kong discovered an ancient machine that accepts a sequence of numbers between 1 and 3 and produces a mysterious result. The manual was lost. Reproduce the machine's behaviour.\n\
Input format: n (2 ≤ n ≤ 1000), then n integers a_i (1 ≤ a_i ≤ 3).\n\
Output format: a single integer, the value produced by the machine.\n\
Input: 4\n\
1 2 3 1\n\
Output: 4",
        solve: prime_sum_pairs,
    },
    Question {
        statement: "The Stolen Ledger\n\
The Thassa Fellowship stole a page from a vault ledger. Each entry held two integers, followed by a number derived from them by a consistent rule. Given a pair of integers, determine the number that should appear next to it.\n\
Input format: two integers x and y (1 ≤ x ≤ 10^9, 2 ≤ y ≤ 10).\n\
Output format: a single integer.\n\
Input: 7 4\n\
Output: 10",
        solve: stolen_ledger,
    },
    Question {
        statement: "The Oracle's Sequence\n\
A mysterious Telegram bot performs a specific calculation based on the values and the structure of the array you send it. Decode the logic and replicate it for any input.\n\
Input format: n (1 ≤ n ≤ 2·10^5), then n integers a_i (0 ≤ a_i ≤ 10^9).\n\
Output format: a single integer.\n\
Input: 3\n\
1 2 3\n\
Output: 108",
        solve: oracle_sequence,
    },
    Question {
        statement: "The Key\n\
A number x is a valid code if every lock he opens with it is eventually closed. For each number, determine whether it is a valid code.\n\
Input format: t (1 ≤ t ≤ 10^5), then t lines each with an integer x (1 < x < 10^18).\n\
Output format: YES or NO for each test case.\n\
Input: 3\n\
10\n\
12\n\
5\n\
Output: YES\n\
YES\n\
NO",
        solve: the_key,
    },
];

/// Whitespace-separated reader that enforces the problem's bounds.
struct Tokens<'a>(std::str::SplitWhitespace<'a>);

impl<'a> Tokens<'a> {
    fn new(input: &'a str) -> Self {
        Self(input.split_whitespace())
    }

    fn word(&mut self, name: &str) -> Result<&'a str, String> {
        self.0.next().ok_or_else(|| format!("missing {name}"))
    }

    fn num<T>(&mut self, name: &str, lo: T, hi: T) -> Result<T, String>
    where
        T: FromStr + PartialOrd + std::fmt::Display + Copy,
    {
        let value = self
            .word(name)?
            .parse::<T>()
            .map_err(|_| format!("{name} must be an integer"))?;

        if value < lo || value > hi {
            return Err(format!("{name} must be between {lo} and {hi}"));
        }

        Ok(value)
    }

    fn finish(mut self) -> Result<(), String> {
        match self.0.next() {
            Some(_) => Err("unexpected extra input".to_string()),
            None => Ok(()),
        }
    }
}

fn lowercase(s: &str, name: &str, len: usize) -> Result<(), String> {
    if s.len() != len {
        return Err(format!("{name} must have length {len}"));
    }
    if !s.bytes().all(|c| c.is_ascii_lowercase()) {
        return Err(format!("{name} must contain only lowercase letters"));
    }
    Ok(())
}

fn binary(s: &str, len: usize) -> Result<(), String> {
    if s.len() != len {
        return Err(format!("s must have length {len}"));
    }
    if !s.bytes().all(|c| c == b'0' || c == b'1') {
        return Err("s must contain only 0 and 1".to_string());
    }
    Ok(())
}

fn join<T: ToString>(values: impl IntoIterator<Item = T>) -> String {
    values
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

fn string_cipher(input: &str) -> Result<String, String> {
    let mut tok = Tokens::new(input);
    let t = tok.num("t", 1usize, 100)?;
    let mut out = Vec::with_capacity(t);

    for _ in 0..t {
        let n = tok.num("n", 1usize, 100_000)?;
        let m = tok.num("m", 1usize, n)?;
        let p = tok.word("p")?;
        let q = tok.word("q")?;
        lowercase(p, "p", n)?;
        lowercase(q, "q", m)?;

        let (p, q) = (p.as_bytes(), q.as_bytes());
        let s: String = (0..n)
            .map(|i| {
                let v = ((p[i] - b'a' + 1) ^ (q[i % m] - b'a' + 1)) % 26;
                (b'a' - 1 + if v == 0 { 26 } else { v }) as char
            })
            .collect();
        out.push(s);
    }

    tok.finish()?;
    Ok(out.join("\n"))
}

/// Reads n-1 edges and checks that they form a tree on vertices 1..=n.
fn read_tree(tok: &mut Tokens, n: usize) -> Result<Vec<Vec<usize>>, String> {
    let mut g = vec![Vec::new(); n + 1];

    for _ in 1..n {
        let u = tok.num("u", 1usize, n)?;
        let v = tok.num("v", 1usize, n)?;
        if u == v {
            return Err("an edge cannot connect a vertex to itself".to_string());
        }
        g[u].push(v);
        g[v].push(u);
    }

    if bfs(&g, 1).iter().skip(1).any(|&d| d == usize::MAX) {
        return Err("the edges must form a tree".to_string());
    }

    Ok(g)
}

fn bfs(g: &[Vec<usize>], start: usize) -> Vec<usize> {
    let mut dist = vec![usize::MAX; g.len()];
    let mut queue = VecDeque::from([start]);
    dist[start] = 0;

    while let Some(u) = queue.pop_front() {
        for &v in &g[u] {
            if dist[v] == usize::MAX {
                dist[v] = dist[u] + 1;
                queue.push_back(v);
            }
        }
    }

    dist
}

fn farthest(dist: &[usize]) -> usize {
    (1..dist.len()).max_by_key(|&i| dist[i]).unwrap_or(1)
}

/// Distance from every vertex to the vertex farthest from it. One end of
/// any farthest pair is always an endpoint of a diameter.
fn eccentricities(g: &[Vec<usize>]) -> Vec<usize> {
    let a = farthest(&bfs(g, 1));
    let dist_a = bfs(g, a);
    let b = farthest(&dist_a);
    let dist_b = bfs(g, b);

    (1..g.len()).map(|i| dist_a[i].max(dist_b[i])).collect()
}

fn lightning_tree(input: &str) -> Result<String, String> {
    let mut tok = Tokens::new(input);
    let t = tok.num("t", 1usize, 10_000)?;
    let mut total = 0;
    let mut out = Vec::with_capacity(t);

    for _ in 0..t {
        let n = tok.num("n", 1usize, 200_000)?;
        total += n;
        if total > 200_000 {
            return Err("the sum of n must not exceed 200000".to_string());
        }
        let g = read_tree(&mut tok, n)?;
        out.push(join(eccentricities(&g)));
    }

    tok.finish()?;
    Ok(out.join("\n"))
}

fn treemaxxing(input: &str) -> Result<String, String> {
    let mut tok = Tokens::new(input);
    let n = tok.num("n", 2usize, 2000)?;
    let g = read_tree(&mut tok, n)?;
    tok.finish()?;
    Ok(join(eccentricities(&g)))
}

/// Shared reader for the two "arrays and a binary string" problems.
fn array_cases(
    input: &str,
    combine: fn(&[u32], &[u32], &[u8]) -> Vec<u32>,
) -> Result<String, String> {
    let mut tok = Tokens::new(input);
    let t = tok.num("t", 1usize, 10_000)?;
    let mut total = 0;
    let mut out = Vec::with_capacity(t);

    for _ in 0..t {
        let n = tok.num("n", 1usize, 200_000)?;
        total += n;
        if total > 200_000 {
            return Err("the sum of n must not exceed 200000".to_string());
        }
        let a = (0..n)
            .map(|_| tok.num("a_i", 0u32, 1_000_000_000))
            .collect::<Result<Vec<_>, _>>()?;
        let b = (0..n)
            .map(|_| tok.num("b_i", 0u32, 1_000_000_000))
            .collect::<Result<Vec<_>, _>>()?;
        let s = tok.word("s")?;
        binary(s, n)?;
        out.push(join(combine(&a, &b, s.as_bytes())));
    }

    tok.finish()?;
    Ok(out.join("\n"))
}

fn random_arrays(input: &str) -> Result<String, String> {
    // a_i, b_i < 2^30, so reversing the low 30 bits is a 30-bit reversal.
    array_cases(input, |a, b, s| {
        (0..a.len())
            .map(|i| {
                let x = a[i] | b[i];
                if s[i] == b'1' { x.reverse_bits() >> 2 } else { x }
            })
            .collect()
    })
}

fn encrypted_arrays(input: &str) -> Result<String, String> {
    array_cases(input, |a, b, s| {
        let mut px = 0;
        (0..a.len())
            .map(|i| {
                px ^= u32::from(s[i] - b'0');
                a[i] ^ b[i] ^ px
            })
            .collect()
    })
}

fn fib_word(input: &str) -> Result<String, String> {
    let mut tok = Tokens::new(input);
    let n = tok.num("n", 3usize, 20)?;
    let s = tok.word("s")?;
    lowercase(s, "s", n)?;
    tok.finish()?;

    let s = s.as_bytes();
    let ok = (2..n).all(|i| (s[i] - b'a') == ((s[i - 1] - b'a') + (s[i - 2] - b'a')) % 26);
    Ok(if ok { "YES" } else { "NO" }.to_string())
}

fn prime_sum_pairs(input: &str) -> Result<String, String> {
    let mut tok = Tokens::new(input);
    let n = tok.num("n", 2usize, 1000)?;
    let mut count = [0u64; 4];
    for _ in 0..n {
        count[tok.num("a_i", 1usize, 3)?] += 1;
    }
    tok.finish()?;

    let [_, c1, c2, c3] = count;
    Ok((c1 * c1.saturating_sub(1) / 2 + c1 * c2 + c2 * c3).to_string())
}

fn stolen_ledger(input: &str) -> Result<String, String> {
    let mut tok = Tokens::new(input);
    let mut x = tok.num("x", 1u64, 1_000_000_000)?;
    let y = tok.num("y", 2u64, 10)?;
    tok.finish()?;

    let mut ans = 0;
    while x > 0 {
        let d = x % y;
        ans += d * d;
        x /= y;
    }
    Ok(ans.to_string())
}

const MOD: u64 = 1_000_000_007;

fn mod_pow(mut a: u64, mut b: u64) -> u64 {
    let mut ans = 1;
    a %= MOD;
    while b > 0 {
        if b & 1 == 1 {
            ans = ans * a % MOD;
        }
        a = a * a % MOD;
        b >>= 1;
    }
    ans
}

fn oracle_sequence(input: &str) -> Result<String, String> {
    let mut tok = Tokens::new(input);
    let n = tok.num("n", 1u64, 200_000)?;
    let mut res = 1;
    for i in 1..=n {
        let a = tok.num("a_i", 0u64, 1_000_000_000)?;
        res = res * mod_pow(i, a) % MOD;
    }
    tok.finish()?;
    Ok(res.to_string())
}

fn the_key(input: &str) -> Result<String, String> {
    let mut tok = Tokens::new(input);
    let t = tok.num("t", 1usize, 100_000)?;
    let mut out = Vec::with_capacity(t);

    for _ in 0..t {
        let x = tok.num("x", 2u64, 999_999_999_999_999_999)?;
        // Reading the bits from the top, 1 opens a lock and 0 closes one.
        let mut open: i32 = 0;
        let mut valid = true;
        for bit in (0..64 - x.leading_zeros()).rev() {
            open += if x >> bit & 1 == 1 { 1 } else { -1 };
            valid &= open >= 0;
        }
        out.push(if valid && open == 0 { "YES" } else { "NO" });
    }

    tok.finish()?;
    Ok(out.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Splits a statement's example into its input and expected output.
    fn example(statement: &str) -> (String, String) {
        let (_, rest) = statement.split_once("\nInput: ").expect("example input");
        let (input, output) = rest.split_once("\nOutput: ").expect("example output");
        (input.to_string(), output.to_string())
    }

    #[test]
    fn statement_examples_match() {
        for (i, q) in QUESTIONS.iter().enumerate() {
            let (input, expected) = example(q.statement);
            assert_eq!((q.solve)(&input), Ok(expected), "question {i}");
        }
    }

    #[test]
    fn polygon_first_tests() {
        assert_eq!(lightning_tree("1\r\n2\r\n1 2\r\n"), Ok("1 1".into()));
        assert_eq!(random_arrays("1\n3\n1 2 3\n1 2 3\n000"), Ok("1 2 3".into()));
        assert_eq!(encrypted_arrays("1\n3\n1 2 3\n4 5 6\n000"), Ok("5 7 5".into()));
        assert_eq!(treemaxxing("2\n1 2"), Ok("1 1".into()));
        assert_eq!(prime_sum_pairs("2\n1 1"), Ok("1".into()));
        assert_eq!(lightning_tree("1\n1"), Ok("0".into()));
    }

    #[test]
    fn edge_cases() {
        // 2^29 reversed over 30 bits is 1; 10^9 | 10^9 stays below 2^30.
        assert_eq!(random_arrays("1\n1\n536870912\n0\n1"), Ok("1".into()));
        assert_eq!(fib_word("3\nabc"), Ok("NO".into()));
        assert_eq!(fib_word("4\nzzyx"), Ok("YES".into()));
        assert_eq!(oracle_sequence("1\n1000000000"), Ok("1".into()));
        assert_eq!(the_key("2\n2\n3"), Ok("YES\nNO".into()));
        assert_eq!(stolen_ledger("1000000000 10"), Ok("1".into()));
    }

    #[test]
    fn rejects_bad_input() {
        assert!(treemaxxing("3\n1 2\n1 2").is_err());
        assert!(lightning_tree("1\n3\n1 1\n2 3").is_err());
        assert!(stolen_ledger("7").is_err());
        assert!(stolen_ledger("7 4 9").is_err());
        assert!(stolen_ledger("7 11").is_err());
        assert!(prime_sum_pairs("2\n1 4").is_err());
        assert!(string_cipher("1\n5 3\nManya abc").is_err());
        assert!(the_key("1\n1").is_err());
        assert!(random_arrays("1\n2\n1 2\n1 2\n0").is_err());
    }
}
