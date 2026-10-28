# rcx: User Instructions

You will be given three things by the organizer:

1. The file **`index.html`**
2. The server **IP address** and **port**
3. Your personal **password** (a long string of letters and numbers)

---

## Getting started

1. **Open `index.html`** in your web browser (double-click it).
2. At the top, fill in the connection bar:
   - **IP**: the server address you were given
   - **Port**: the port you were given
   - **Key**: your password (paste it, as it is long)
3. Click **Load** (or press Enter).

If it works, the status at the top shows **connected** and the number of questions, and a numbered list appears on the left.

---

## Trying a question

1. Click a **number** in the list on the left. The question appears on the right, with its input format, output format and an example.
2. Read the statement carefully. It tells you the allowed ranges and exactly how the input must be laid out.
3. Type or paste your input into the **Input** box. Write it in exactly the format the statement describes. For example, if the statement says "t, then t lines each with an integer", enter `t` first, then those lines.
4. Click **Run →**.
5. The result appears in the **Output** box.

Tip: copy the example input from the statement first, run it, and check that you get the example output. Then try your own inputs.

---

## Understanding the output

| What you see | What it means |
|--------------|---------------|
| The answer | Your input was valid, and this is the expected output. |
| `invalid input: ...` | Your input broke a rule in the statement (wrong format, number out of range, missing or extra values). The message says what is wrong. |
| `invalid key` | Your password is wrong, incomplete, or expired. |
| `connection failed` | The IP or port is wrong, or the server is not running. |

---

## Common problems

- **"invalid key"**: Re-copy the whole password, since it is easy to miss part of it. Passwords expire **3 hours after they were issued**, so if yours has expired, ask the organizer for a new one.
- **Cannot connect**: Double-check the IP and port. If the port box is left empty, the page uses port 3000.
- **Too many requests error**: You are sending runs too quickly. Wait a moment and try again.
- **"invalid input: unexpected extra input"**: You entered more values than the format asks for.
- **"invalid input: missing ..."**: You entered fewer values than the format asks for.

---

## Notes

- Spaces and line breaks between values do not matter much. What matters is the order and number of values.
- Your password is personal. Please do not share it.
