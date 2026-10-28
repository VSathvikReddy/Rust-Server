// The CodeDecode problem set (Polygon contest 62114), ported from each
// problem's reference solution. Every hidden function takes the raw input
// exactly as the problem formats it and returns the expected output, or an
// error message when the input breaks the problem's constraints.

use std::str::FromStr;

pub struct Question {
    pub statement: &'static str,
    pub solve: fn(&str) -> Result<String, String>,
}

pub const QUESTIONS: [Question; 10] = [
    Question {
        statement: "Warm Up Problem\n\
Input format: an integer n (1 ≤ n ≤ 10^6).\n\
Output format: the output the bot would produce for n.\n\
Input: 1\n\
Output: 6",
        solve: warm_up,
    },
    Question {
        statement: "Random Arrays\n\
NOTE: This differs from the Polygon statement: here you can only enter one query, not multiple queries at once using a test case variable.\n\
Input format: n (1 ≤ n ≤ 2·10^5), a line of n integers a_i, a line of n integers b_i (0 ≤ a_i, b_i ≤ 31), and a binary string s of length n.\n\
Output format: n integers c_1 ... c_n.\n\
Input: 3\n\
1 2 3\n\
1 2 3\n\
010\n\
Output: 1 29 3",
        solve: random_arrays,
    },
    Question {
        statement: "Aibohphobia\n\
NOTE: This differs from the Polygon statement: here you can only enter one query, not multiple queries at once using a test case variable.\n\
Input format: a string s of lowercase Latin letters (1 ≤ |s| ≤ 2·10^5).\n\
Output format: a single integer, the mystery level of the string.\n\
Input: abca\n\
Output: 2",
        solve: mirror_matches,
    },
    Question {
        statement: "Salad with a Twist\n\
NOTE: This differs from the Polygon statement: here you can only enter one query, not multiple queries at once using a test case variable.\n\
Input format: n and m (1 ≤ m ≤ n ≤ 10^5), then the strings p and q of lengths n and m. p, q and s use lowercase Latin letters only.\n\
Output format: the decoded string s of length n.\n\
Input: 5 3\n\
manya abc\n\
Output: lcmxc",
        solve: string_cipher,
    },
    Question {
        statement: "The Stolen Ledger\n\
Input format: two integers x and y (1 ≤ x ≤ 10^9, 2 ≤ y ≤ 10).\n\
Output format: a single integer.\n\
Input: 7 4\n\
Output: 10",
        solve: stolen_ledger,
    },
    Question {
        statement: "Prime Murmurs\n\
Input format: n (2 ≤ n ≤ 1000), then n integers a_i (1 ≤ a_i ≤ 3).\n\
Output format: a single integer, the value produced by the machine.\n\
Input: 4\n\
1 2 3 1\n\
Output: 4",
        solve: prime_sum_pairs,
    },
    Question {
        statement: "Another Easy Decoding?\n\
Input format: a positive integer n (n ≥ 1).\n\
Output format: a single integer.\n\
Input: 123456\n\
Output: 27",
        solve: led_sticks,
    },
    Question {
        statement: "The Oracle's Sequence\n\
Input format: n (1 ≤ n ≤ 2·10^5), then n integers a_i (0 ≤ a_i ≤ 10^9).\n\
Output format: a single integer.\n\
Input: 3\n\
1 2 3\n\
Output: 108",
        solve: oracle_sequence,
    },
    Question {
        statement: "The Key\n\
NOTE: This differs from the Polygon statement: here you can only enter one query, not multiple queries at once using a test case variable.\n\
Input format: an integer x (1 < x < 10^18).\n\
Output format: YES or NO.\n\
Input: 10\n\
Output: YES",
        solve: the_key,
    },
    Question {
        statement: "Were You Taught Wrong?\n\
Input format: n (3 ≤ n ≤ 10^5), then a line of n integers a_i and a line of n integers b_i — two permutations of 1, 2, ..., n.\n\
Output format: a single integer.\n\
Input: 3\n\
1 2 3\n\
1 2 3\n\
Output: 14",
        solve: taught_wrong,
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
    let n = tok.num("n", 1usize, 100_000)?;
    let m = tok.num("m", 1usize, n)?;
    let p = tok.word("p")?;
    let q = tok.word("q")?;
    lowercase(p, "p", n)?;
    lowercase(q, "q", m)?;
    tok.finish()?;

    let (p, q) = (p.as_bytes(), q.as_bytes());
    Ok((0..n)
        .map(|i| {
            let v = ((p[i] - b'a' + 1) ^ (q[i % m] - b'a' + 1)) % 26;
            (b'a' - 1 + if v == 0 { 26 } else { v }) as char
        })
        .collect())
}

/// Reader for the "arrays and a binary string" problem.
fn array_cases(
    input: &str,
    combine: fn(&[u32], &[u32], &[u8]) -> Vec<u32>,
) -> Result<String, String> {
    let mut tok = Tokens::new(input);
    let n = tok.num("n", 1usize, 200_000)?;
    let a = (0..n)
        .map(|_| tok.num("a_i", 0u32, 31))
        .collect::<Result<Vec<_>, _>>()?;
    let b = (0..n)
        .map(|_| tok.num("b_i", 0u32, 31))
        .collect::<Result<Vec<_>, _>>()?;
    let s = tok.word("s")?;
    binary(s, n)?;
    tok.finish()?;

    Ok(join(combine(&a, &b, s.as_bytes())))
}

fn random_arrays(input: &str) -> Result<String, String> {
    // a_i, b_i < 2^5, so inverting the low 5 bits is 31 ^ x (= 31 - x).
    array_cases(input, |a, b, s| {
        (0..a.len())
            .map(|i| {
                let x = a[i] | b[i];
                if s[i] == b'1' { 31 ^ x } else { x }
            })
            .collect()
    })
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
    let x = tok.num("x", 2u64, 999_999_999_999_999_999)?;
    tok.finish()?;

    // Reading the bits from the top, 1 opens a lock and 0 closes one.
    let mut open: i32 = 0;
    let mut valid = true;
    for bit in (0..64 - x.leading_zeros()).rev() {
        open += if x >> bit & 1 == 1 { 1 } else { -1 };
        valid &= open >= 0;
    }
    Ok(if valid && open == 0 { "YES" } else { "NO" }.to_string())
}

/// Reads n integers in 1..=n and checks that each appears exactly once (1-indexed).
fn read_permutation(tok: &mut Tokens, name: &str, n: usize) -> Result<Vec<usize>, String> {
    let mut p = vec![0; n + 1];
    let mut seen = vec![false; n + 1];
    let elem = format!("{name}_i");

    for i in 1..=n {
        let x = tok.num(&elem, 1usize, n)?;
        if seen[x] {
            return Err(format!("Array '{name}' must contain each number from 1 to {n} exactly once"));
        }
        seen[x] = true;
        p[i] = x;
    }

    Ok(p)
}

fn taught_wrong(input: &str) -> Result<String, String> {
    let mut tok = Tokens::new(input);
    let n = tok.num("n", 3usize, 100_000)?;
    let a = read_permutation(&mut tok, "a", n)?;
    let b = read_permutation(&mut tok, "b", n)?;
    tok.finish()?;

    let ans: u64 = (1..=n).map(|i| (i as u64) * (a[b[i]] as u64)).sum();
    Ok(ans.to_string())
}

fn led_sticks(input: &str) -> Result<String, String> {
    const STICK_VALUES: [u64; 10] = [6, 2, 5, 5, 4, 5, 6, 3, 7, 6];

    let mut tok = Tokens::new(input);
    let s = tok.word("n")?;
    tok.finish()?;

    if s.len() > 100_000 {
        return Err("n must have at most 100000 digits".to_string());
    }
    if !s.bytes().all(|c| c.is_ascii_digit()) {
        return Err("n must be a positive integer".to_string());
    }

    let trimmed = s.trim_start_matches('0');
    if trimmed.is_empty() {
        return Err("n must be at least 1".to_string());
    }

    let mut digit_counts = [0u64; 10];
    let mut total_sticks = 0u64;

    for b in trimmed.bytes() {
        let digit = (b - b'0') as usize;
        digit_counts[digit] += 1;
        total_sticks += digit_counts[digit] * STICK_VALUES[digit];
    }

    Ok(total_sticks.to_string())
}

fn warm_up(input: &str) -> Result<String, String> {
    let mut tok = Tokens::new(input);
    let n = tok.num("n", 1u64, 1_000_000)?;
    tok.finish()?;
    Ok((4 * n + 2).to_string())
}

/// Number of positions where a string agrees with its reverse.
fn mirror_matches(input: &str) -> Result<String, String> {
    let mut tok = Tokens::new(input);
    let s = tok.word("s")?;
    tok.finish()?;
    if s.len() > 200_000 {
        return Err("s must have at most 200000 letters".to_string());
    }
    lowercase(s, "s", s.len())?;

    let s = s.as_bytes();
    let n = s.len();
    Ok((0..n).filter(|&i| s[i] == s[n - 1 - i]).count().to_string())
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
        assert_eq!(random_arrays("3\n1 2 3\n1 2 3\n000"), Ok("1 2 3".into()));
        assert_eq!(prime_sum_pairs("2\n1 1"), Ok("1".into()));
        assert_eq!(taught_wrong("3\n2 3 1\n3 1 2"), Ok("14".into()));
        assert_eq!(led_sticks("00123"), Ok("12".into()));
        assert_eq!(mirror_matches("a"), Ok("1".into()));
        assert_eq!(warm_up("999999"), Ok("3999998".into()));
        assert_eq!(mirror_matches("racecar"), Ok("7".into()));
    }

    #[test]
    fn edge_cases() {
        // Inverting the low 5 bits: 31 | 0 becomes 0, and 0 | 0 becomes 31.
        assert_eq!(random_arrays("3\n31 0 16\n0 0 8\n110"), Ok("0 31 24".into()));
        assert_eq!(oracle_sequence("1\n1000000000"), Ok("1".into()));
        assert_eq!(the_key("2"), Ok("YES".into()));
        assert_eq!(the_key("3"), Ok("NO".into()));
        assert_eq!(stolen_ledger("1000000000 10"), Ok("1".into()));
        assert_eq!(led_sticks("10"), Ok("8".into()));
        assert_eq!(led_sticks("11"), Ok("6".into()));
    }

    #[test]
    fn rejects_bad_input() {
        assert!(stolen_ledger("7").is_err());
        assert!(stolen_ledger("7 4 9").is_err());
        assert!(stolen_ledger("7 11").is_err());
        assert!(prime_sum_pairs("2\n1 4").is_err());
        assert!(string_cipher("5 3\nManya abc").is_err());
        assert!(the_key("1").is_err());
        assert!(random_arrays("2\n1 2\n1 2\n0").is_err());
        assert!(random_arrays("1\n32\n0\n0").is_err());
        assert!(random_arrays("1\n0\n32\n1").is_err());
        assert!(taught_wrong("2\n1 2\n1 2").is_err());
        assert!(taught_wrong("3\n1 2 2\n1 2 3").is_err());
        assert!(led_sticks("0").is_err());
        assert!(led_sticks("000").is_err());
        assert!(led_sticks("-5").is_err());
        assert!(led_sticks("12a").is_err());
        assert!(mirror_matches("Ab").is_err());
        assert!(warm_up("0").is_err());
        assert!(warm_up("1000001").is_err());
        assert!(warm_up("1 2").is_err());
        assert!(mirror_matches("ab cd").is_err());
        // The old multi-case format (a leading t) is rejected.
        assert!(mirror_matches("1\nabc").is_err());
        assert!(the_key("2\n10\n12").is_err());
        assert!(string_cipher("1\n5 3\nmanya abc").is_err());
        assert!(random_arrays("1\n3\n1 2 3\n1 2 3\n010").is_err());
    }
}