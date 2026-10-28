import csv
import smtplib
from email.message import EmailMessage

# Configuration
SMTP_SERVER = "smtp.gmail.com"
SMTP_PORT = 465
SENDER_EMAIL = "programmingclubiitm@gmail.com"
SENDER_PASSWORD = (
    "wpys lcoh fpdz ipgm"
)


def get_content(team_name, password):
    return f"""Hi {team_name},

Thank you for participating in Code Decode, our flagship reverse coding challenge. 

Your server pass key for the competition is provided below:
Pass Key: {password}

Contest Link: https://codeforces.com/contestInvitation/ddce5c12dcfb63f030e5b476752969cfb16ee98b
Wishing your team the very best of luck in the challenge!

Best regards,  
Programming Club IITM"""

# Connect to Gmail SMTP and send
with smtplib.SMTP_SSL(SMTP_SERVER, SMTP_PORT) as server:
  server.login(SENDER_EMAIL, SENDER_PASSWORD)

  with open("out4.csv", mode="r", encoding="utf-8") as file:
    reader = csv.DictReader(file)

    for row in reader:
      # Automatically fetch the email and password from your CSV columns
      mail = row.get("mail_id") or row.get("mail") or row.get("email")
      team_name = row.get("Team Name")
      password = row.get("Password")

      msg = EmailMessage()
      msg["Subject"] = (
          "Pass Key for Code Decode"
      )
      msg["From"] = SENDER_EMAIL
      msg["To"] = mail

      # Professional Email Body
      msg.set_content(get_content(team_name,password))

      server.send_message(msg)
      print(f"Sent successfully to {mail}")
