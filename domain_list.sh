cd /tmp
curl -LO https://tranco-list.eu/top-1m.csv.zip

CFG=/home/beytullah/Desktop/rust/homoglyph-guard

mkdir -p "$CFG"
unzip -p top-1m.csv.zip | tr -d '\r' | cut -d, -f2 | grep -viE '\.cn$' > "$CFG/popular_domains.txt"

head -3 "$CFG/popular_domains.txt"
wc -l "$CFG/popular_domains.txt"
