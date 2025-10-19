import time
import threading
import http.server
import socketserver
import os
from datetime import datetime

# Configuration
PORT = 8000
HTML_FILE = "index.html"
SVG_FILE = "favicon.svg"
DEVICE_FILE = "/sys/bus/w1/devices/28-000001cda180/w1_slave"
UPDATE_INTERVAL = 10  # Update HTML every 1 second
CURRENT_DATE="---"
def read_temp():
    global CURRENT_DATE
    now = datetime.now()
# Get timezone abbreviation like CEST
    tz = time.strftime('%Z')
    today_text = now.strftime(f"%a %b %d %I:%M:%S %p {tz} %Y")
#today_text = date.today().strftime("%B %d, %Y")  # Example: 'September 17, 2025'
    CURRENT_DATE = f"&laquo;{today_text}&raquo;"  # Fixed date as per query

    try:
        with open(DEVICE_FILE, 'r') as f:
            lines = f.readlines()
        if lines[0].strip().endswith('YES'):
            temp_str = lines[1].split('t=')[1].strip()
            temp_c = float(temp_str) / 1000.0
            return temp_c
        else:
            return None
    except Exception as e:
        print(f"Error reading temperature: {e}")
        return None

def generate_html(temp):
    formatted_temp = "N/A" if temp is None else f"{temp:.3f}&deg;C"
    html_content = f"""
<!DOCTYPE html>
<html lang="en">
<head>
    <title>{formatted_temp} at {CURRENT_DATE} on FrambOS</title>
	<meta http-equiv="refresh" content="10"> 
<link rel="icon" href="/favicon.ico" sizes="any">
<link rel="icon" href="/favicon.svg" type="image/svg+xml">
</head>
<body>
    <h1>Temperature on FrambOS</h1>
On FrambOS right now, the date is {CURRENT_DATE} and the temperature is {formatted_temp}
</body>
</html>
    """
    with open(HTML_FILE, 'w') as f:
        f.write(html_content)
    svg_content = f"""
<svg height="32" width="32" xmlns="http://www.w3.org/2000/svg">
  <text x="0" y="24" fill="red">{formatted_temp[:5]}</text>
</svg>
    """
    with open(SVG_FILE, 'w') as f:
        f.write(svg_content)

def update_html():
    while True:
        temp = read_temp()
        generate_html(temp)
        #print(f"Temperature: {temp:.3f} °C" if temp is not None else "Failed to read temperature")
        time.sleep(UPDATE_INTERVAL)

def start_server():
    Handler = http.server.SimpleHTTPRequestHandler
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(("", PORT), Handler) as httpd:
        print(f"Serving at http://192.168.0.2:{PORT}")
        httpd.serve_forever()

if __name__ == "__main__":
    # Start HTML update in a separate thread
    update_thread = threading.Thread(target=update_html, daemon=True)
    update_thread.start()
    # Start web server
    start_server()
