#!/usr/bin/env python3
"""Build Kirano's starter grocery shelf the way Khaynjo's dish shelf was built.

Khaynjo keeps read-only catalogue art as WebP only
(`assets/items/<folder>/<name>.webp`). Photos are fetched as JPEG/PNG
(Wikimedia Commons thumbs) and re-encoded to WebP before they are stored.
This script does that for a neighbourhood kirana list: one folder per
aisle, one WebP per item, plus `assets/templates/neighbourhood/template.json`
(English name, Hindi alias, starter rupee price, unit, `shelf:` photo).

Prices are starter counter figures, not a legal MRP. Re-run is safe:
an existing valid WebP is left alone.

    python3 apps/kirano/scripts/build-shelf.py
"""

from __future__ import annotations

import io
import json
import time
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SHELF = ROOT / "assets" / "items"
TEMPLATE_DIR = ROOT / "assets" / "templates" / "neighbourhood"
UA = "KiranoCatalogue/1.0 (https://kirano.shop; starter shelf; attribution in SOURCES.jsonl)"
COMMONS = "https://commons.wikimedia.org/w/api.php"

# folder, slug, English name, Hindi alias, rupees, unit, search query
# Hindi may be empty. Query is what Commons is asked for (generic, not a brand).
ITEMS: list[tuple[str, str, str, str, int, str, str]] = []


def add(folder: str, rows: list[tuple]) -> None:
    for row in rows:
        slug, name, hindi, price, unit, query = row
        ITEMS.append((folder, slug, name, hindi, price, unit, query))


add("staples", [
    ("sona-masoori-rice", "Sona masoori rice", "सोना मसूरी चावल", 60, "kg", "sona masoori rice"),
    ("basmati-rice", "Basmati rice", "बासमती चावल", 140, "kg", "basmati rice"),
    ("kolam-rice", "Kolam rice", "कोल्हम चावल", 70, "kg", "kolam rice"),
    ("brown-rice", "Brown rice", "भूरा चावल", 90, "kg", "brown rice"),
    ("boiled-rice", "Boiled rice", "उसना चावल", 55, "kg", "parboiled rice"),
    ("broken-rice", "Broken rice", "खिचड़ी चावल", 40, "kg", "broken rice"),
    ("idli-rice", "Idli rice", "इडली चावल", 65, "kg", "idli rice"),
    ("poha-thick", "Thick poha", "मोटा पोहा", 50, "kg", "poha flattened rice"),
    ("poha-thin", "Thin poha", "पतला पोहा", 55, "kg", "thin poha"),
    ("murmura", "Puffed rice", "मुरमुरा", 40, "kg", "puffed rice murmura"),
    ("sabudana", "Sago", "साबूदाना", 80, "kg", "sabudana sago"),
    ("wheat-atta", "Wheat atta", "गेहूं का आटा", 45, "kg", "whole wheat flour atta"),
    ("multigrain-atta", "Multigrain atta", "मल्टीग्रेन आटा", 70, "kg", "multigrain flour"),
    ("maida", "Maida", "मैदा", 42, "kg", "maida all purpose flour"),
    ("rava", "Rava", "रवा", 48, "kg", "semolina rava"),
    ("besan", "Besan", "बेसन", 90, "kg", "besan gram flour"),
    ("rice-flour", "Rice flour", "चावल का आटा", 50, "kg", "rice flour"),
    ("corn-flour", "Corn flour", "कॉर्न फ्लोर", 60, "kg", "corn flour"),
    ("ragi-flour", "Ragi flour", "रागी आटा", 70, "kg", "ragi finger millet flour"),
    ("jowar-flour", "Jowar flour", "ज्वार आटा", 55, "kg", "jowar sorghum flour"),
    ("bajra-flour", "Bajra flour", "बाजरा आटा", 50, "kg", "bajra pearl millet flour"),
    ("makki-atta", "Makki atta", "मक्की का आटा", 48, "kg", "cornmeal makki flour"),
    ("daliya", "Daliya", "दलिया", 45, "kg", "broken wheat daliya"),
    ("oats", "Oats", "ओट्स", 40, "pc", "rolled oats"),
    ("vermicelli", "Vermicelli", "सेवई", 30, "pc", "vermicelli seviyan"),
    ("sabudana-nylon", "Nylon sago", "नायलॉन साबूदाना", 90, "kg", "nylon sabudana"),
    ("pressed-rice-red", "Red poha", "लाल पोहा", 70, "kg", "red flattened rice"),
    ("quinoa", "Quinoa", "क्विनोआ", 180, "pc", "quinoa grain"),
    ("corn-flakes", "Corn flakes", "कॉर्न फ्लेक्स", 45, "pc", "corn flakes cereal"),
    ("muesli", "Muesli", "म्यूसली", 120, "pc", "muesli"),
    ("wheat-bran", "Wheat bran", "गेहूं का चोकर", 35, "kg", "wheat bran"),
    ("suji-coarse", "Coarse suji", "मोटा सूजी", 46, "kg", "coarse semolina"),
    ("arrowroot", "Arrowroot", "अरारोट", 80, "pc", "arrowroot powder"),
    ("tapioca", "Tapioca", "टैपिओका", 70, "kg", "tapioca"),
    ("barley", "Barley", "जौ", 50, "kg", "barley grain"),
    ("sorghum-grain", "Jowar grain", "ज्वार दाना", 40, "kg", "sorghum grain"),
    ("pearl-millet", "Bajra grain", "बाजरा दाना", 38, "kg", "pearl millet grain"),
    ("finger-millet", "Ragi grain", "रागी दाना", 55, "kg", "finger millet grain"),
    ("foxtail-millet", "Foxtail millet", "कंगनी", 80, "kg", "foxtail millet"),
    ("little-millet", "Little millet", "कुटकी", 75, "kg", "little millet"),
])

add("pulses", [
    ("toor-dal", "Toor dal", "अरहर दाल", 140, "kg", "toor dal pigeon pea"),
    ("moong-dal", "Moong dal", "मूंग दाल", 120, "kg", "moong dal"),
    ("moong-whole", "Whole moong", "साबुत मूंग", 110, "kg", "whole mung bean"),
    ("masoor-dal", "Masoor dal", "मसूर दाल", 100, "kg", "red lentil masoor"),
    ("masoor-whole", "Whole masoor", "साबुत मसूर", 95, "kg", "brown lentil"),
    ("chana-dal", "Chana dal", "चना दाल", 90, "kg", "chana dal"),
    ("kabuli-chana", "Kabuli chana", "काबुली चना", 110, "kg", "chickpea kabuli"),
    ("kala-chana", "Kala chana", "काला चना", 85, "kg", "black chickpea"),
    ("urad-dal", "Urad dal", "उड़द दाल", 130, "kg", "urad dal black gram"),
    ("urad-whole", "Whole urad", "साबुत उड़द", 120, "kg", "whole black gram"),
    ("urad-chilka", "Urad chilka", "उड़द छिलका", 125, "kg", "split black gram"),
    ("rajma", "Rajma", "राजमा", 140, "kg", "rajma kidney beans"),
    ("lobia", "Lobia", "लोबिया", 90, "kg", "black eyed peas"),
    ("moth", "Moth beans", "मोठ", 80, "kg", "moth beans"),
    ("horse-gram", "Horse gram", "कुलथी", 90, "kg", "horse gram"),
    ("green-peas-dry", "Dry green peas", "सूखा मटर", 80, "kg", "dried green peas"),
    ("soy-chunks", "Soy chunks", "सोया बड़ी", 40, "pc", "soy chunks"),
    ("soy-beans", "Soy beans", "सोयाबीन", 90, "kg", "soybeans"),
    ("val", "Field beans", "वाल", 100, "kg", "field beans hyacinth"),
    ("matki", "Matki", "मटकी", 95, "kg", "moth bean sprouts"),
    ("moong-chilka", "Moong chilka", "मूंग छिलका", 115, "kg", "split mung bean"),
    ("toor-oily", "Oily toor", "तेली अरहर", 145, "kg", "oily toor dal"),
    ("roasted-chana", "Roasted chana", "भुना चना", 30, "pc", "roasted chickpeas"),
    ("chana-sattu", "Sattu", "सत्तू", 50, "kg", "sattu roasted gram flour"),
    ("mixed-dal", "Mixed dal", "मिक्स दाल", 110, "kg", "mixed lentils"),
    ("white-peas", "White peas", "सफेद मटर", 70, "kg", "dried white peas"),
    ("cowpeas", "Cowpeas", "चौला", 90, "kg", "cowpeas"),
    ("pinto-beans", "Pinto beans", "पिंटो बीन्स", 160, "kg", "pinto beans"),
])

add("oils-ghee", [
    ("sunflower-oil", "Sunflower oil", "सूरजमुखी तेल", 140, "L", "sunflower cooking oil"),
    ("groundnut-oil", "Groundnut oil", "मूंगफली तेल", 180, "L", "peanut oil"),
    ("mustard-oil", "Mustard oil", "सरसों तेल", 160, "L", "mustard oil"),
    ("coconut-oil", "Coconut oil", "नारियल तेल", 200, "L", "coconut oil"),
    ("sesame-oil", "Sesame oil", "तिल तेल", 280, "L", "sesame oil"),
    ("rice-bran-oil", "Rice bran oil", "राइस ब्रान तेल", 150, "L", "rice bran oil"),
    ("soybean-oil", "Soybean oil", "सोया तेल", 130, "L", "soybean oil"),
    ("olive-oil", "Olive oil", "जैतून तेल", 450, "pc", "olive oil bottle"),
    ("ghee", "Ghee", "घी", 550, "kg", "ghee clarified butter"),
    ("vanaspati", "Vanaspati", "वनस्पति", 120, "kg", "vanaspati"),
    ("cow-ghee", "Cow ghee", "गाय का घी", 700, "kg", "cow ghee"),
    ("palm-oil", "Palm oil", "पाम तेल", 110, "L", "palm oil"),
    ("flaxseed-oil", "Flaxseed oil", "अलसी तेल", 250, "pc", "flaxseed oil"),
    ("castor-oil", "Castor oil", "अरंडी तेल", 80, "pc", "castor oil bottle"),
    ("refined-oil-pouch", "Refined oil pouch", "रिफाइंड तेल पाउच", 20, "pc", "cooking oil pouch"),
    ("ghee-jar", "Ghee jar", "घी का डिब्बा", 280, "pc", "ghee jar"),
])

add("spices", [
    ("turmeric-powder", "Turmeric powder", "हल्दी पाउडर", 40, "pc", "turmeric powder"),
    ("turmeric-whole", "Turmeric fingers", "हल्दी गांठ", 30, "pc", "fresh turmeric root"),
    ("red-chilli-powder", "Red chilli powder", "लाल मिर्च पाउडर", 50, "pc", "red chilli powder"),
    ("kashmiri-chilli", "Kashmiri chilli", "कश्मीरी मिर्च", 70, "pc", "kashmiri chilli"),
    ("green-chilli", "Green chilli", "हरी मिर्च", 40, "kg", "green chilli"),
    ("coriander-powder", "Coriander powder", "धनिया पाउडर", 40, "pc", "coriander powder"),
    ("coriander-seeds", "Coriander seeds", "साबुत धनिया", 30, "pc", "coriander seeds"),
    ("cumin-seeds", "Cumin seeds", "जीरा", 60, "pc", "cumin seeds"),
    ("cumin-powder", "Cumin powder", "जीरा पाउडर", 50, "pc", "cumin powder"),
    ("mustard-seeds", "Mustard seeds", "राई", 25, "pc", "mustard seeds"),
    ("fenugreek-seeds", "Fenugreek seeds", "मेथी दाना", 20, "pc", "fenugreek seeds"),
    ("kasuri-methi", "Kasuri methi", "कसूरी मेथी", 20, "pc", "dried fenugreek leaves"),
    ("garam-masala", "Garam masala", "गरम मसाला", 40, "pc", "garam masala"),
    ("black-pepper", "Black pepper", "काली मिर्च", 80, "pc", "black peppercorns"),
    ("pepper-powder", "Pepper powder", "काली मिर्च पाउडर", 70, "pc", "black pepper powder"),
    ("cardamom-green", "Green cardamom", "हरी इलायची", 120, "pc", "green cardamom"),
    ("cardamom-black", "Black cardamom", "बड़ी इलायची", 80, "pc", "black cardamom"),
    ("cinnamon", "Cinnamon", "दालचीनी", 40, "pc", "cinnamon sticks"),
    ("cloves", "Cloves", "लौंग", 50, "pc", "cloves spice"),
    ("bay-leaf", "Bay leaf", "तेज पत्ता", 15, "pc", "bay leaves"),
    ("star-anise", "Star anise", "चक्र फूल", 40, "pc", "star anise"),
    ("fennel-seeds", "Fennel seeds", "सौंफ", 30, "pc", "fennel seeds"),
    ("ajwain", "Ajwain", "अजवाइन", 25, "pc", "ajwain carom seeds"),
    ("asafoetida", "Asafoetida", "हींग", 40, "pc", "asafoetida hing"),
    ("dry-ginger", "Dry ginger", "सौंठ", 40, "pc", "dried ginger"),
    ("ginger-fresh", "Fresh ginger", "अदरक", 80, "kg", "fresh ginger"),
    ("garlic", "Garlic", "लहसुन", 120, "kg", "garlic bulbs"),
    ("onion", "Onion", "प्याज", 30, "kg", "onion"),
    ("tamarind", "Tamarind", "इमली", 40, "pc", "tamarind"),
    ("kokum", "Kokum", "कोकम", 40, "pc", "kokum"),
    ("curry-leaves", "Curry leaves", "कढ़ी पत्ता", 10, "pc", "curry leaves"),
    ("coriander-leaves", "Coriander leaves", "हरा धनिया", 20, "pc", "coriander leaves"),
    ("mint-leaves", "Mint", "पुदीना", 15, "pc", "mint leaves"),
    ("mango-powder", "Amchur", "अमचूर", 35, "pc", "amchur mango powder"),
    ("chaat-masala", "Chaat masala", "चाट मसाला", 30, "pc", "chaat masala"),
    ("pav-bhaji-masala", "Pav bhaji masala", "पाव भाजी मसाला", 30, "pc", "pav bhaji masala"),
    ("sambar-powder", "Sambar powder", "सांबर पाउडर", 40, "pc", "sambar powder"),
    ("rasam-powder", "Rasam powder", "रसम पाउडर", 35, "pc", "rasam powder"),
    ("kitchen-king", "Kitchen masala", "किचन मसाला", 40, "pc", "indian spice blend"),
    ("meat-masala", "Meat masala", "मीट मसाला", 40, "pc", "meat masala"),
    ("chicken-masala", "Chicken masala", "चिकन मसाला", 40, "pc", "chicken masala powder"),
    ("chole-masala", "Chole masala", "छोले मसाला", 30, "pc", "chole masala"),
    ("biryani-masala", "Biryani masala", "बिरयानी मसाला", 40, "pc", "biryani masala"),
    ("white-pepper", "White pepper", "सफेद मिर्च", 90, "pc", "white pepper"),
    ("nutmeg", "Nutmeg", "जायफल", 40, "pc", "nutmeg"),
    ("mace", "Mace", "जावित्री", 50, "pc", "mace spice"),
    ("poppy-seeds", "Poppy seeds", "खसखस", 80, "pc", "poppy seeds"),
    ("sesame-seeds", "Sesame seeds", "तिल", 40, "pc", "sesame seeds"),
    ("black-sesame", "Black sesame", "काला तिल", 50, "pc", "black sesame seeds"),
    ("nigella", "Kalonji", "कलौंजी", 30, "pc", "nigella seeds"),
    ("carom-whole", "Ajwain whole", "अजवाइन साबुत", 20, "pc", "carom seeds"),
    ("pomegranate-seeds", "Anardana", "अनारदाना", 50, "pc", "dried pomegranate seeds"),
    ("saffron", "Saffron", "केसर", 200, "pc", "saffron strands"),
    ("rock-salt", "Rock salt", "सेंधा नमक", 20, "pc", "rock salt"),
    ("black-salt", "Black salt", "काला नमक", 20, "pc", "black salt kala namak"),
    ("chat-masala-small", "Chaat masala sachet", "चाट मसाला पुड़िया", 5, "pc", "spice sachet"),
    ("whole-red-chilli", "Whole red chilli", "सूखी लाल मिर्च", 40, "pc", "dried red chillies"),
    ("green-cardamom-powder", "Cardamom powder", "इलायची पाउडर", 90, "pc", "cardamom powder"),
    ("jaljeera", "Jaljeera powder", "जलजीरा", 30, "pc", "jaljeera powder"),
    ("pickle-masala", "Pickle masala", "अचार मसाला", 35, "pc", "pickle masala"),
    ("god-masala", "Goda masala", "गोडा मसाला", 60, "pc", "maharashtrian goda masala"),
    ("pepper-long", "Long pepper", "पीपली", 40, "pc", "long pepper"),
    ("dry-mango", "Dry mango", "सूखा आम", 50, "pc", "dried mango"),
    ("asafoetida-compound", "Compound hing", "कम्पाउंड हींग", 25, "pc", "hing powder"),
    ("lemon", "Lemon", "नींबू", 60, "kg", "lemon"),
    ("ginger-garlic-paste", "Ginger garlic paste", "अदरक लहसुन पेस्ट", 30, "pc", "ginger garlic paste"),
])

add("salt-sugar", [
    ("iodised-salt", "Iodised salt", "नमक", 22, "kg", "iodised salt"),
    ("crystal-salt", "Crystal salt", "खड़ा नमक", 18, "kg", "coarse salt"),
    ("sugar", "Sugar", "चीनी", 45, "kg", "white sugar"),
    ("brown-sugar", "Brown sugar", "ब्राउन शुगर", 80, "kg", "brown sugar"),
    ("jaggery", "Jaggery", "गुड़", 60, "kg", "jaggery gur"),
    ("jaggery-powder", "Jaggery powder", "गुड़ पाउडर", 70, "kg", "jaggery powder"),
    ("mishri", "Mishri", "मिश्री", 80, "kg", "rock candy mishri"),
    ("honey", "Honey", "शहद", 120, "pc", "honey jar"),
    ("sugar-free", "Sugar substitute", "शुगर फ्री", 90, "pc", "sugar substitute tablets"),
    ("palm-sugar", "Palm sugar", "ताड़ गुड़", 90, "kg", "palm sugar"),
    ("icing-sugar", "Icing sugar", "आइसिंग शुगर", 40, "pc", "icing sugar"),
    ("salt-sachet", "Salt sachet", "नमक की पुड़िया", 2, "pc", "salt packet"),
])

add("tea-coffee", [
    ("tea-dust", "Tea dust", "चाय पत्ती", 40, "pc", "black tea leaves"),
    ("tea-leaf", "Tea leaf", "चाय की पत्ती", 80, "pc", "loose leaf tea"),
    ("green-tea", "Green tea", "ग्रीन टी", 90, "pc", "green tea"),
    ("masala-tea", "Masala tea", "मसाला चाय", 70, "pc", "masala chai"),
    ("instant-coffee", "Instant coffee", "इंस्टेंट कॉफी", 90, "pc", "instant coffee"),
    ("filter-coffee", "Filter coffee", "फिल्टर कॉफी", 120, "pc", "south indian filter coffee"),
    ("coffee-beans", "Coffee beans", "कॉफी बीन्स", 200, "pc", "coffee beans"),
    ("chicory", "Chicory", "चिकोरी", 60, "pc", "chicory coffee"),
    ("cocoa-powder", "Cocoa powder", "कोको पाउडर", 80, "pc", "cocoa powder"),
    ("drinking-chocolate", "Drinking chocolate", "ड्रिंकिंग चॉकलेट", 70, "pc", "drinking chocolate powder"),
    ("tea-bags", "Tea bags", "टी बैग", 60, "pc", "tea bags"),
    ("herbal-tea", "Herbal tea", "हर्बल चाय", 80, "pc", "herbal tea"),
    ("lemon-tea", "Lemon tea", "लेमन टी", 50, "pc", "lemon tea"),
    ("coffee-premix", "Coffee premix", "कॉफी प्रीमिक्स", 10, "pc", "instant coffee sachet"),
    ("tea-premix", "Tea premix", "चाय प्रीमिक्स", 10, "pc", "instant tea sachet"),
    ("sugar-candy-tea", "Elaichi tea", "इलायची चाय", 75, "pc", "cardamom tea"),
])

add("biscuits", [
    ("glucose-biscuit", "Glucose biscuit", "ग्लूकोज बिस्कुट", 10, "pc", "glucose biscuits"),
    ("marie-biscuit", "Marie biscuit", "मैरी बिस्कुट", 15, "pc", "marie biscuits"),
    ("cream-biscuit", "Cream biscuit", "क्रीम बिस्कुट", 20, "pc", "cream sandwich biscuit"),
    ("salted-cracker", "Salted cracker", "नमकीन क्रैकर", 20, "pc", "salted crackers"),
    ("digestive-biscuit", "Digestive biscuit", "डाइजेस्टिव बिस्कुट", 30, "pc", "digestive biscuits"),
    ("oat-biscuit", "Oat biscuit", "ओट बिस्कुट", 30, "pc", "oat biscuits"),
    ("cookie", "Cookie", "कुकी", 40, "pc", "chocolate chip cookie"),
    ("rusk", "Rusk", "रस्क", 30, "pc", "cake rusk"),
    ("khari", "Khari", "खारी", 25, "pc", "khari puff pastry"),
    ("jeera-biscuit", "Jeera biscuit", "जीरा बिस्कुट", 20, "pc", "cumin biscuit"),
    ("coconut-biscuit", "Coconut biscuit", "नारियल बिस्कुट", 20, "pc", "coconut biscuit"),
    ("chocolate-biscuit", "Chocolate biscuit", "चॉकलेट बिस्कुट", 20, "pc", "chocolate biscuit"),
    ("wafer", "Wafer", "वेफर", 10, "pc", "wafer biscuit"),
    ("nice-biscuit", "Nice biscuit", "नाइस बिस्कुट", 15, "pc", "coconut nice biscuit"),
    ("arrowroot-biscuit", "Arrowroot biscuit", "अरारोट बिस्कुट", 20, "pc", "arrowroot biscuit"),
    ("milk-biscuit", "Milk biscuit", "मिल्क बिस्कुट", 10, "pc", "milk biscuits"),
    ("cashew-cookie", "Cashew cookie", "काजू कुकी", 40, "pc", "cashew cookies"),
    ("butter-cookie", "Butter cookie", "बटर कुकी", 35, "pc", "butter cookies"),
    ("multigrain-biscuit", "Multigrain biscuit", "मल्टीग्रेन बिस्कुट", 30, "pc", "multigrain biscuit"),
    ("protein-biscuit", "Protein biscuit", "प्रोटीन बिस्कुट", 20, "pc", "high protein biscuit"),
    ("toast-biscuit", "Toast", "टोस्ट", 25, "pc", "bread toast rusk"),
    ("nankhatai", "Nankhatai", "नानखटाई", 40, "pc", "nankhatai"),
    ("osmania-biscuit", "Osmania biscuit", "उस्मानिया", 20, "pc", "osmania biscuit"),
    ("fruit-biscuit", "Fruit biscuit", "फ्रूट बिस्कुट", 15, "pc", "fruit biscuit"),
])

add("snacks", [
    ("potato-chips", "Potato chips", "चिप्स", 20, "pc", "potato chips"),
    ("banana-chips", "Banana chips", "केले के चिप्स", 40, "pc", "banana chips"),
    ("namkeen-mixture", "Mixture", "मिक्सचर", 40, "pc", "indian namkeen mixture"),
    ("bhujia", "Bhujia", "भुजिया", 40, "pc", "bhujia sev"),
    ("sev", "Sev", "सेव", 30, "pc", "nylon sev"),
    ("chakli", "Chakli", "chakli", 50, "pc", "chakli snack"),
    ("murukku", "Murukku", "मुरुक्कु", 40, "pc", "murukku"),
    ("chivda", "Chivda", "चिवड़ा", 40, "pc", "poha chivda"),
    ("peanuts", "Peanuts", "मूंगफली", 40, "pc", "roasted peanuts"),
    ("salted-peanuts", "Salted peanuts", "नमकीन मूंगफली", 20, "pc", "salted peanuts"),
    ("masala-peanuts", "Masala peanuts", "मसाला मूंगफली", 20, "pc", "masala coated peanuts"),
    ("popcorn", "Popcorn", "पॉपकॉर्न", 20, "pc", "popcorn"),
    ("makhana", "Makhana", "मखाना", 80, "pc", "fox nuts makhana"),
    ("soya-sticks", "Soya sticks", "सोया स्टिक", 20, "pc", "soya sticks snack"),
    ("kurkure-style", "Corn puffs", "कॉर्न पफ्स", 10, "pc", "corn puffs snack"),
    ("nachos", "Nachos", "नाचोस", 30, "pc", "nachos"),
    ("papad", "Papad", "पापड़", 40, "pc", "papadum"),
    ("masala-papad", "Masala papad", "मसाला पापड़", 10, "pc", "masala papad"),
    ("rice-papad", "Rice papad", "चावल पापड़", 40, "pc", "rice papad"),
    ("appalam", "Appalam", "अप्पलम", 40, "pc", "appalam"),
    ("khakhra", "Khakhra", "खाखरा", 40, "pc", "khakhra"),
    ("mathri", "Mathri", "mathri", 40, "pc", "mathri snack"),
    ("dry-fruit-mix", "Trail mix", "ड्राई फ्रूट मिक्स", 80, "pc", "trail mix"),
    ("roasted-chana-pack", "Roasted chana pack", "भुना चना पैक", 10, "pc", "roasted chana packet"),
    ("moong-dal-namkeen", "Moong dal namkeen", "मूंग दाल नमकीन", 30, "pc", "fried moong dal"),
    ("aloo-bhujia", "Aloo bhujia", "आलू भुजिया", 20, "pc", "aloo bhujia"),
    ("corn-flakes-mixture", "Corn flakes mixture", "कॉर्न फ्लेक्स मिक्सचर", 40, "pc", "cornflake mixture"),
    ("baked-namkeen", "Baked namkeen", "बेक्ड नमकीन", 30, "pc", "baked namkeen"),
    ("extruded-rings", "Snack rings", "स्नैक रिंग्स", 10, "pc", "potato rings snack"),
    ("salted-cashew-pack", "Salted cashew pack", "नमकीन काजू", 50, "pc", "salted cashew"),
    ("flavoured-makhana", "Flavoured makhana", "मसाला मखाना", 40, "pc", "roasted fox nuts"),
    ("rice-crisps", "Rice crisps", "राइस क्रिस्प", 20, "pc", "rice crackers"),
    ("chakli-pack", "Chakli pack", "चकली पैक", 30, "pc", "chakli packet"),
    ("farsan", "Farsan", "फरसाण", 50, "pc", "gujarati farsan"),
    ("ganthia", "Ganthia", "गांठिया", 40, "pc", "ganthia"),
    ("sing-bhujia", "Sing bhujia", "सिंग भुजिया", 30, "pc", "peanut bhujia"),
])

add("noodles-ready", [
    ("instant-noodles", "Instant noodles", "नूडल्स", 14, "pc", "instant noodles"),
    ("atta-noodles", "Atta noodles", "आटा नूडल्स", 20, "pc", "whole wheat noodles"),
    ("cup-noodles", "Cup noodles", "कप नूडल्स", 40, "pc", "cup noodles"),
    ("hakka-noodles", "Hakka noodles", "हक्का नूडल्स", 30, "pc", "hakka noodles dry"),
    ("pasta", "Pasta", "पास्ता", 40, "pc", "dry pasta"),
    ("macaroni", "Macaroni", "मैकरोनी", 35, "pc", "macaroni"),
    ("spaghetti", "Spaghetti", "स्पेगेटी", 40, "pc", "spaghetti"),
    ("instant-upma", "Instant upma", "इंस्टेंट उपमा", 20, "pc", "instant upma mix"),
    ("instant-poha", "Instant poha", "इंस्टेंट पोहा", 20, "pc", "instant poha mix"),
    ("soup-sachet", "Soup sachet", "सूप", 10, "pc", "instant soup sachet"),
    ("pasta-sauce", "Pasta sauce", "पास्ता सॉस", 60, "pc", "pasta sauce jar"),
    ("oats-masala", "Masala oats", "मसाला ओट्स", 30, "pc", "masala oats"),
])

add("breakfast-mixes", [
    ("idli-mix", "Idli mix", "इडली मिक्स", 40, "pc", "idli batter mix"),
    ("dosa-mix", "Dosa mix", "डोसा मिक्स", 40, "pc", "dosa mix"),
    ("dhokla-mix", "Dhokla mix", "ढोकला मिक्स", 30, "pc", "dhokla mix"),
    ("gulab-jamun-mix", "Gulab jamun mix", "गुलाब जामुन मिक्स", 40, "pc", "gulab jamun mix"),
    ("cake-mix", "Cake mix", "केक मिक्स", 50, "pc", "cake mix"),
    ("pancake-mix", "Pancake mix", "पैनकेक मिक्स", 60, "pc", "pancake mix"),
    ("custard-powder", "Custard powder", "कस्टर्ड पाउडर", 30, "pc", "custard powder"),
    ("jelly-crystals", "Jelly crystals", "जेली", 20, "pc", "jelly crystals"),
    ("baking-powder", "Baking powder", "बेकिंग पाउडर", 20, "pc", "baking powder"),
    ("baking-soda", "Baking soda", "बेकिंग सोडा", 15, "pc", "baking soda"),
    ("yeast", "Yeast", "यीस्ट", 20, "pc", "dry yeast"),
    ("cocoa-baking", "Baking cocoa", "बेकिंग कोको", 80, "pc", "unsweetened cocoa"),
])

add("dairy", [
    ("toned-milk", "Toned milk", "टोन्ड दूध", 28, "pc", "milk pouch"),
    ("full-cream-milk", "Full cream milk", "फुल क्रीम दूध", 32, "pc", "full cream milk"),
    ("curd", "Curd", "दही", 30, "pc", "curd dahi"),
    ("paneer", "Paneer", "पनीर", 90, "pc", "paneer"),
    ("butter", "Butter", "मक्खन", 55, "pc", "butter"),
    ("cheese-slice", "Cheese slice", "चीज़ स्लाइस", 80, "pc", "cheese slices"),
    ("cheese-block", "Cheese block", "चीज़ ब्लॉक", 110, "pc", "cheddar cheese"),
    ("milk-powder", "Milk powder", "दूध पाउडर", 40, "pc", "milk powder"),
    ("condensed-milk", "Condensed milk", "कंडेंस्ड मिल्क", 50, "pc", "condensed milk"),
    ("buttermilk", "Buttermilk", "छाछ", 15, "pc", "buttermilk chaas"),
    ("lassi", "Lassi", "लस्सी", 20, "pc", "lassi"),
    ("flavoured-milk", "Flavoured milk", "फ्लेवर्ड मिल्क", 25, "pc", "flavoured milk"),
    ("cream", "Fresh cream", "क्रीम", 40, "pc", "fresh cream"),
    ("khoya", "Khoya", "खोया", 200, "kg", "khoya mawa"),
    ("shrikhand", "Shrikhand", "श्रीखंड", 40, "pc", "shrikhand"),
    ("yogurt-cup", "Yogurt cup", "योगर्ट", 30, "pc", "yogurt cup"),
    ("dairy-whitener", "Dairy whitener", "डेयरी व्हाइटनर", 50, "pc", "dairy whitener"),
    ("malai-paneer", "Malai paneer", "मलाई पनीर", 100, "pc", "malai paneer"),
])

add("bakery", [
    ("white-bread", "White bread", "ब्रेड", 40, "pc", "sliced white bread"),
    ("brown-bread", "Brown bread", "ब्राउन ब्रेड", 45, "pc", "brown bread"),
    ("milk-bread", "Milk bread", "मिल्क ब्रेड", 40, "pc", "milk bread"),
    ("pav", "Pav", "पाव", 20, "pc", "pav bread rolls"),
    ("bun", "Bun", "बन", 10, "pc", "burger bun"),
    ("burger-bun", "Burger bun", "बर्गर बन", 15, "pc", "sesame burger bun"),
    ("pizza-base", "Pizza base", "पिज़्ज़ा बेस", 30, "pc", "pizza base"),
    ("croissant", "Croissant", "क्रोइसां", 30, "pc", "croissant"),
    ("muffin", "Muffin", "मफिन", 25, "pc", "muffin"),
    ("cake-slice", "Cake slice", "केक", 30, "pc", "slice of cake"),
    ("fruit-bread", "Fruit bread", "फ्रूट ब्रेड", 40, "pc", "fruit bread"),
    ("garlic-bread", "Garlic bread", "गार्लिक ब्रेड", 40, "pc", "garlic bread"),
    ("multigrain-bread", "Multigrain bread", "मल्टीग्रेन ब्रेड", 50, "pc", "multigrain bread"),
    ("kulcha", "Kulcha", "कुलचा", 15, "pc", "kulcha"),
    ("naan-pack", "Naan pack", "नान", 30, "pc", "naan bread"),
    ("tortilla", "Tortilla", "टॉर्टिया", 40, "pc", "flour tortilla"),
])

add("beverages", [
    ("packaged-water", "Packaged water", "पानी", 20, "pc", "bottled drinking water"),
    ("soda", "Soda", "सोडा", 20, "pc", "soda water bottle"),
    ("cola", "Cola", "कोला", 40, "pc", "cola drink"),
    ("orange-drink", "Orange drink", "ऑरेंज ड्रिंक", 20, "pc", "orange soft drink"),
    ("lemon-drink", "Lemon drink", "नींबू ड्रिंक", 20, "pc", "lemon soda"),
    ("mango-drink", "Mango drink", "आम का जूस", 20, "pc", "mango drink carton"),
    ("fruit-juice", "Fruit juice", "फ्रूट जूस", 30, "pc", "fruit juice carton"),
    ("coconut-water", "Coconut water", "नारियल पानी", 40, "pc", "packaged coconut water"),
    ("energy-drink", "Energy drink", "एनर्जी ड्रिंक", 100, "pc", "energy drink can"),
    ("glucose-drink", "Glucose drink", "ग्लूकोज", 30, "pc", "glucose powder drink"),
    ("squash", "Squash", "squash", 80, "pc", "fruit squash"),
    ("rose-syrup", "Rose syrup", "गुलाब शरबत", 60, "pc", "rose syrup"),
    ("sharbat", "Sharbat", "शरबत", 50, "pc", "sharbat concentrate"),
    ("buttermilk-pack", "Buttermilk pack", "छाछ पैक", 15, "pc", "packaged buttermilk"),
    ("cold-coffee", "Cold coffee", "कोल्ड कॉफी", 30, "pc", "cold coffee bottle"),
    ("iced-tea", "Iced tea", "आइस टी", 30, "pc", "iced tea bottle"),
    ("tonic-water", "Tonic water", "टॉनिक वाटर", 40, "pc", "tonic water"),
    ("sparkling-water", "Sparkling water", "स्पार्कलिंग पानी", 30, "pc", "sparkling water"),
    ("apple-juice", "Apple juice", "सेब का जूस", 40, "pc", "apple juice"),
    ("mixed-fruit", "Mixed fruit drink", "मिक्स फ्रूट", 25, "pc", "mixed fruit juice"),
    ("electrolyte-drink", "Electrolyte drink", "इलेक्ट्रोलाइट", 35, "pc", "electrolyte drink"),
    ("lassi-bottle", "Lassi bottle", "लस्सी बोतल", 25, "pc", "bottled lassi"),
    ("milkshake", "Milkshake", "मिल्कशेक", 30, "pc", "milkshake bottle"),
    ("nimbu-pani", "Nimbu pani", "नींबू पानी", 15, "pc", "lemon drink packet"),
])

add("personal-care", [
    ("bathing-soap", "Bathing soap", "साबुन", 30, "pc", "bathing soap bar"),
    ("beauty-soap", "Beauty soap", "ब्यूटी सोप", 40, "pc", "beauty soap"),
    ("glycerin-soap", "Glycerin soap", "ग्लिसरीन सोप", 35, "pc", "glycerin soap"),
    ("shampoo-sachet", "Shampoo sachet", "शैम्पू", 2, "pc", "shampoo sachet"),
    ("shampoo-bottle", "Shampoo bottle", "शैम्पू बोतल", 120, "pc", "shampoo bottle"),
    ("hair-oil", "Hair oil", "हेयर ऑयल", 60, "pc", "hair oil bottle"),
    ("coconut-hair-oil", "Coconut hair oil", "नारियल हेयर ऑयल", 50, "pc", "coconut hair oil"),
    ("toothpaste", "Toothpaste", "टूथपेस्ट", 50, "pc", "toothpaste"),
    ("toothbrush", "Toothbrush", "टूथब्रश", 25, "pc", "toothbrush"),
    ("mouthwash", "Mouthwash", "माउथवॉश", 90, "pc", "mouthwash"),
    ("talcum", "Talcum powder", "टाल्कम", 60, "pc", "talcum powder"),
    ("face-cream", "Face cream", "फेस क्रीम", 80, "pc", "face cream"),
    ("face-wash", "Face wash", "फेस वॉश", 90, "pc", "face wash"),
    ("body-lotion", "Body lotion", "बॉडी लोशन", 120, "pc", "body lotion"),
    ("petroleum-jelly", "Petroleum jelly", "पेट्रोलियम जेली", 40, "pc", "petroleum jelly"),
    ("lip-balm", "Lip balm", "लिप बाम", 30, "pc", "lip balm"),
    ("razor", "Razor", "रेजर", 40, "pc", "safety razor"),
    ("shaving-cream", "Shaving cream", "शेविंग क्रीम", 50, "pc", "shaving cream"),
    ("aftershave", "Aftershave", "आफ्टरशेव", 80, "pc", "aftershave"),
    ("deodorant", "Deodorant", "डियोड्रेंट", 150, "pc", "deodorant spray"),
    ("sanitary-pad", "Sanitary pad", "सैनिटरी पैड", 40, "pc", "sanitary pads"),
    ("cotton-buds", "Cotton buds", "कॉटन बड्स", 30, "pc", "cotton buds"),
    ("hair-comb", "Comb", "कंघी", 20, "pc", "hair comb"),
    ("hair-clip", "Hair clip", "हेयर क्लिप", 15, "pc", "hair clip"),
    ("nail-cutter", "Nail cutter", "नाखून कटर", 25, "pc", "nail clipper"),
    ("hand-wash", "Hand wash", "हैंड वॉश", 70, "pc", "liquid hand wash"),
    ("hand-sanitizer", "Hand sanitizer", "सैनिटाइजर", 50, "pc", "hand sanitizer"),
    ("sunscreen", "Sunscreen", "सनस्क्रीन", 180, "pc", "sunscreen lotion"),
    ("kajal", "Kajal", "काजल", 20, "pc", "kajal"),
    ("bindi", "Bindi", "बिंदी", 10, "pc", "bindi pack"),
    ("hair-colour", "Hair colour", "हेयर कलर", 40, "pc", "hair colour cream"),
    ("conditioner", "Conditioner", "कंडीशनर", 120, "pc", "hair conditioner"),
    ("body-wash", "Body wash", "बॉडी वॉश", 140, "pc", "body wash"),
    ("baby-soap", "Baby soap", "बेबी सोप", 40, "pc", "baby soap"),
    ("baby-oil", "Baby oil", "बेबी ऑयल", 70, "pc", "baby oil"),
    ("baby-powder", "Baby powder", "बेबी पाउडर", 60, "pc", "baby powder"),
    ("diaper", "Diaper", "डायपर", 15, "pc", "baby diaper"),
    ("wet-wipes", "Wet wipes", "wet wipes", 40, "pc", "wet wipes"),
    ("tissue", "Tissue", "टिशू", 30, "pc", "facial tissue"),
    ("toilet-roll", "Toilet roll", "टॉयलेट रोल", 30, "pc", "toilet paper roll"),
])

add("home-care", [
    ("detergent-bar", "Detergent bar", "डिटर्जेंट बार", 10, "pc", "laundry soap bar"),
    ("detergent-powder", "Detergent powder", "डिटर्जेंट पाउडर", 40, "pc", "laundry detergent powder"),
    ("liquid-detergent", "Liquid detergent", "लिक्विड डिटर्जेंट", 120, "pc", "liquid laundry detergent"),
    ("fabric-conditioner", "Fabric conditioner", "फैब्रिक कंडीशनर", 90, "pc", "fabric softener"),
    ("dishwash-bar", "Dishwash bar", "बर्तन बार", 10, "pc", "dishwash bar"),
    ("dishwash-liquid", "Dishwash liquid", "बर्तन लिक्विड", 70, "pc", "dishwashing liquid"),
    ("floor-cleaner", "Floor cleaner", "फ्लोर क्लीनर", 80, "pc", "floor cleaner"),
    ("phenyl", "Phenyl", "फिनाइल", 40, "pc", "phenyl disinfectant"),
    ("toilet-cleaner", "Toilet cleaner", "टॉयलेट क्लीनर", 80, "pc", "toilet cleaner"),
    ("glass-cleaner", "Glass cleaner", "ग्लास क्लीनर", 70, "pc", "glass cleaner"),
    ("bleach", "Bleach", "ब्लीच", 40, "pc", "laundry bleach"),
    ("scrubber", "Scrubber", "स्क्रबर", 15, "pc", "dish scrubber"),
    ("steel-wool", "Steel wool", "स्टील वूल", 20, "pc", "steel wool"),
    ("broom", "Broom", "झाड़ू", 60, "pc", "broom"),
    ("mop", "Mop", "पोंछा", 120, "pc", "floor mop"),
    ("dustbin-bag", "Dustbin bag", "कचरे की थैली", 30, "pc", "garbage bags"),
    ("aluminium-foil", "Aluminium foil", "एल्युमिनियम फॉइल", 50, "pc", "aluminium foil"),
    ("cling-film", "Cling film", "क्लिंग फिल्म", 40, "pc", "cling film"),
    ("tissue-kitchen", "Kitchen tissue", "किचन टिशू", 40, "pc", "kitchen paper towel"),
    ("matchbox", "Matchbox", "माचिस", 2, "pc", "matchbox"),
    ("candle", "Candle", "मोमबत्ती", 10, "pc", "candle"),
    ("mosquito-coil", "Mosquito coil", "मच्छर कॉइल", 30, "pc", "mosquito coil"),
    ("mosquito-liquid", "Mosquito liquid", "मच्छर लिक्विड", 80, "pc", "mosquito repellent liquid"),
    ("insect-spray", "Insect spray", "कीटनाशक", 120, "pc", "insect spray"),
    ("naphthalene", "Naphthalene", "नेफ्थलीन", 20, "pc", "naphthalene balls"),
    ("room-freshener", "Room freshener", "रूम फ्रेशनर", 90, "pc", "room freshener"),
    ("sponge", "Sponge", "स्पंज", 15, "pc", "cleaning sponge"),
    ("cloth-pin", "Cloth pin", "क्लिप", 20, "pc", "clothes pegs"),
])

add("pooja", [
    ("agarbatti", "Agarbatti", "अगरबत्ती", 20, "pc", "incense sticks"),
    ("dhoop", "Dhoop", "धूप", 20, "pc", "dhoop incense"),
    ("camphor", "Camphor", "कपूर", 20, "pc", "camphor"),
    ("cotton-wicks", "Cotton wicks", "बत्ती", 10, "pc", "cotton wicks diya"),
    ("kumkum", "Kumkum", "कुमकुम", 15, "pc", "kumkum"),
    ("haldi-kumkum", "Haldi kumkum", "हल्दी कुमकुम", 20, "pc", "haldi kumkum"),
    ("ganga-jal", "Ganga jal", "गंगा जल", 20, "pc", "holy water bottle"),
    ("puja-oil", "Puja oil", "पूजा तेल", 30, "pc", "lamp oil"),
    ("diya", "Diya", "दिया", 10, "pc", "clay diya"),
    ("coconut-whole", "Coconut", "नारियल", 30, "pc", "whole coconut"),
    ("betel-leaf", "Betel leaf", "पान", 5, "pc", "betel leaves"),
    ("supari", "Supari", "सुपारी", 20, "pc", "betel nut"),
    ("roli", "Roli", "रोली", 10, "pc", "roli powder"),
    ("kalava", "Kalava", "कलावा", 10, "pc", "sacred thread mauli"),
])

add("stationery", [
    ("notebook", "Notebook", "कॉपी", 30, "pc", "school notebook"),
    ("pen", "Pen", "पेन", 10, "pc", "ballpoint pen"),
    ("pencil", "Pencil", "पेंसिल", 5, "pc", "pencil"),
    ("eraser", "Eraser", "रबर", 5, "pc", "eraser"),
    ("sharpener", "Sharpener", "शार्पनर", 5, "pc", "pencil sharpener"),
    ("scale", "Scale", "स्केल", 10, "pc", "ruler"),
    ("geometry-box", "Geometry box", "ज्योमेट्री बॉक्स", 40, "pc", "geometry box"),
    ("crayon", "Crayon", "क्रेयॉन", 30, "pc", "crayon set"),
    ("glue", "Glue", "गोंद", 15, "pc", "glue stick"),
    ("tape", "Tape", "टेप", 20, "pc", "adhesive tape"),
    ("envelope", "Envelope", "लिफाफा", 2, "pc", "paper envelope"),
    ("chart-paper", "Chart paper", "चार्ट पेपर", 10, "pc", "chart paper"),
    ("stapler", "Stapler", "स्टेपलर", 40, "pc", "stapler"),
    ("marker", "Marker", "मार्कर", 20, "pc", "marker pen"),
])

add("pickles", [
    ("mango-pickle", "Mango pickle", "आम का अचार", 60, "pc", "mango pickle"),
    ("lime-pickle", "Lime pickle", "नींबू का अचार", 50, "pc", "lime pickle"),
    ("mixed-pickle", "Mixed pickle", "मिक्स अचार", 55, "pc", "mixed pickle"),
    ("chilli-pickle", "Chilli pickle", "मिर्च का अचार", 50, "pc", "green chilli pickle"),
    ("garlic-pickle", "Garlic pickle", "लहसुन का अचार", 60, "pc", "garlic pickle"),
    ("murabba", "Murabba", "मुरब्बा", 50, "pc", "murabba"),
    ("chutney", "Chutney", "चटनी", 40, "pc", "mango chutney"),
    ("tomato-ketchup", "Tomato ketchup", "टमाटर केचप", 70, "pc", "tomato ketchup"),
    ("sauce", "Chilli sauce", "चिली सॉस", 40, "pc", "chilli sauce"),
    ("soya-sauce", "Soya sauce", "सोया सॉस", 40, "pc", "soy sauce"),
    ("vinegar", "Vinegar", "सिरका", 25, "pc", "vinegar"),
    ("jam", "Jam", "जैम", 60, "pc", "fruit jam"),
    ("peanut-butter", "Peanut butter", "पीनट बटर", 90, "pc", "peanut butter"),
    ("mayonnaise", "Mayonnaise", "मेयोनीज", 70, "pc", "mayonnaise"),
    ("schezwan", "Schezwan sauce", "शेजवान सॉस", 50, "pc", "schezwan sauce"),
    ("mint-chutney", "Mint chutney", "पुदीना चटनी", 30, "pc", "mint chutney"),
    ("tamarind-chutney", "Imli chutney", "इमली चटनी", 30, "pc", "tamarind chutney"),
    ("pickle-mango-kg", "Mango pickle kg", "आम अचार किलो", 180, "kg", "homemade mango pickle"),
])

add("dry-fruits", [
    ("almonds", "Almonds", "बादाम", 200, "pc", "almonds"),
    ("cashews", "Cashews", "काजू", 220, "pc", "cashews"),
    ("raisins", "Raisins", "किशमिश", 80, "pc", "raisins"),
    ("pistachios", "Pistachios", "पिस्ता", 250, "pc", "pistachios"),
    ("walnuts", "Walnuts", "अखरोट", 200, "pc", "walnuts"),
    ("dates", "Dates", "खजूर", 80, "pc", "dates fruit"),
    ("figs", "Figs", "अंजीर", 150, "pc", "dried figs"),
    ("apricots", "Apricots", "खुबानी", 160, "pc", "dried apricots"),
    ("pumpkin-seeds", "Pumpkin seeds", "कद्दू के बीज", 80, "pc", "pumpkin seeds"),
    ("sunflower-seeds", "Sunflower seeds", "सूरजमुखी बीज", 50, "pc", "sunflower seeds"),
    ("flax-seeds", "Flax seeds", "अलसी", 40, "pc", "flax seeds"),
    ("chia-seeds", "Chia seeds", "चिया सीड्स", 80, "pc", "chia seeds"),
    ("coconut-dry", "Dry coconut", "सूखा नारियल", 40, "pc", "dried coconut"),
    ("copra", "Copra", "कोपरा", 50, "pc", "copra"),
    ("desiccated-coconut", "Desiccated coconut", "सुखा नारियल बुरादा", 40, "pc", "desiccated coconut"),
    ("mixed-dry-fruit", "Mixed dry fruit", "मिक्स ड्राई फ्रूट", 180, "pc", "mixed dry fruits"),
    ("anjeer", "Anjeer", "अंजीर पैक", 90, "pc", "anjeer figs"),
    ("munakka", "Munakka", "मुनक्का", 90, "pc", "munakka raisins"),
    ("charoli", "Charoli", "चिरौंजी", 120, "pc", "charoli seeds"),
    ("melon-seeds", "Melon seeds", "खरबूजे के बीज", 60, "pc", "melon seeds magaz"),
    ("pistachio-kernel", "Pista kernel", "पिस्ता गिरी", 280, "pc", "pistachio kernels"),
    ("almond-flakes", "Almond flakes", "बादाम कतरन", 90, "pc", "almond flakes"),
])

add("produce", [
    ("potato", "Potato", "आलू", 25, "kg", "potato"),
    ("tomato", "Tomato", "टमाटर", 30, "kg", "tomato"),
    ("onion-red", "Red onion", "लाल प्याज", 30, "kg", "red onion"),
    ("cabbage", "Cabbage", "पत्ता गोभी", 25, "kg", "cabbage"),
    ("cauliflower", "Cauliflower", "फूल गोभी", 40, "pc", "cauliflower"),
    ("carrot", "Carrot", "गाजर", 40, "kg", "carrot"),
    ("beans", "French beans", "फ्रेंच बीन्स", 60, "kg", "french beans"),
    ("peas", "Green peas", "मटर", 50, "kg", "green peas"),
    ("spinach", "Spinach", "पालक", 20, "pc", "spinach"),
    ("methi-leaves", "Methi leaves", "मेथी", 15, "pc", "fenugreek leaves"),
    ("brinjal", "Brinjal", "बैंगन", 40, "kg", "eggplant"),
    ("okra", "Okra", "भिंडी", 50, "kg", "okra bhindi"),
    ("bottle-gourd", "Bottle gourd", "लौकी", 30, "kg", "bottle gourd"),
    ("ridge-gourd", "Ridge gourd", "तोरई", 40, "kg", "ridge gourd"),
    ("bitter-gourd", "Bitter gourd", "करेला", 40, "kg", "bitter gourd"),
    ("cucumber", "Cucumber", "खीरा", 30, "kg", "cucumber"),
    ("capsicum", "Capsicum", "शिमला मिर्च", 60, "kg", "capsicum"),
    ("green-peas-fresh", "Fresh peas", "ताजा मटर", 40, "kg", "peas in pod"),
    ("banana", "Banana", "केला", 40, "kg", "banana"),
    ("apple", "Apple", "सेब", 120, "kg", "apple fruit"),
    ("orange", "Orange", "संतरा", 60, "kg", "orange fruit"),
    ("pomegranate", "Pomegranate", "अनार", 120, "kg", "pomegranate"),
    ("grapes", "Grapes", "अंगूर", 80, "kg", "grapes"),
    ("papaya", "Papaya", "पपीता", 40, "kg", "papaya"),
    ("watermelon", "Watermelon", "तरबूज", 25, "kg", "watermelon"),
    ("mango", "Mango", "आम", 80, "kg", "mango fruit"),
    ("sapota", "Chikoo", "चीकू", 60, "kg", "sapodilla chikoo"),
    ("guava", "Guava", "अमरूद", 50, "kg", "guava"),
])

add("eggs-bread-more", [
    ("eggs", "Eggs", "अंडे", 7, "pc", "chicken eggs"),
    ("brown-eggs", "Brown eggs", "ब्राउन अंडे", 9, "pc", "brown eggs"),
    ("bread-crumbs", "Bread crumbs", "ब्रेड क्रम्ब्स", 30, "pc", "bread crumbs"),
    ("corn-kernels", "Sweet corn", "स्वीट कॉर्न", 30, "pc", "sweet corn kernels"),
    ("frozen-peas", "Frozen peas", "फ्रोजन मटर", 40, "pc", "frozen peas"),
    ("ice-cream", "Ice cream", "आइसक्रीम", 40, "pc", "ice cream cup"),
    ("ice-cream-stick", "Ice cream stick", "आइसक्रीम स्टिक", 20, "pc", "ice cream bar"),
    ("curd-pouch", "Curd pouch", "दही पाउच", 20, "pc", "curd packet"),
])

add("canned", [
    ("baked-beans", "Baked beans", "बेक्ड बीन्स", 50, "pc", "baked beans can"),
    ("sweet-corn-can", "Sweet corn can", "स्वीट कॉर्न कैन", 40, "pc", "canned sweet corn"),
    ("mushroom-can", "Mushroom can", "मशरूम कैन", 50, "pc", "canned mushrooms"),
    ("tomato-puree", "Tomato puree", "टमाटर प्यूरी", 30, "pc", "tomato puree"),
    ("coconut-milk", "Coconut milk", "कोकोनट मिल्क", 60, "pc", "coconut milk can"),
    ("olives", "Olives", "ऑलिव", 90, "pc", "olives jar"),
    ("tuna", "Tuna", "टूना", 120, "pc", "canned tuna"),
    ("sardine", "Sardine", "सार्डिन", 40, "pc", "canned sardines"),
    ("fruit-cocktail", "Fruit cocktail", "फ्रूट कॉकटेल", 70, "pc", "canned fruit cocktail"),
    ("pineapple-can", "Pineapple can", "अनानास कैन", 60, "pc", "canned pineapple"),
    ("condensed-milk-tin", "Condensed milk tin", "दूध की टिन", 55, "pc", "condensed milk tin"),
    ("ghee-tin", "Ghee tin", "घी टिन", 500, "pc", "ghee tin"),
])


def good_title(title: str) -> bool:
    t = title.lower()
    bad = (
        "formula", "structural", "skeletal", "diagram", "chembox", "ball-and-stick",
        "3d", "crystal structure", "acid crp", "lewis", "spacefill", "svg",
    )
    return not any(b in t for b in bad)


def commons_hits(query: str) -> list[dict]:
    params = {
        "action": "query",
        "format": "json",
        "generator": "search",
        "gsrsearch": query,
        "gsrnamespace": "6",
        "gsrlimit": "8",
        "prop": "imageinfo",
        "iiprop": "url|mime",
        "iiurlwidth": "800",
    }
    url = COMMONS + "?" + urllib.parse.urlencode(params)
    req = urllib.request.Request(url, headers={"User-Agent": UA, "Accept": "application/json"})
    with urllib.request.urlopen(req, timeout=25) as resp:
        data = json.load(resp)
    pages = (data.get("query") or {}).get("pages") or {}
    hits = []
    for page in pages.values():
        info = (page.get("imageinfo") or [None])[0] or {}
        mime = (info.get("mime") or "").lower()
        thumb = info.get("thumburl") or ""
        if mime not in {"image/jpeg", "image/png", "image/webp"}:
            continue
        if not good_title(page.get("title") or ""):
            continue
        if not thumb.startswith("http"):
            continue
        hits.append({
            "title": page.get("title") or "",
            "thumb": thumb,
            "page": "https://commons.wikimedia.org/wiki/" + urllib.parse.quote((page.get("title") or "").replace(" ", "_")),
            "mime": mime,
        })
    return hits


def to_webp(raw: bytes, dest: Path) -> None:
    im = Image.open(io.BytesIO(raw))
    im = im.convert("RGB")
    im.thumbnail((800, 800))
    dest.parent.mkdir(parents=True, exist_ok=True)
    im.save(dest, "WEBP", quality=80, method=4)


def fetch_one(item: tuple, folder_query: str) -> dict:
    folder, slug, name, hindi, price, unit, query = item
    dest = SHELF / folder / f"{slug}.webp"
    if dest.is_file() and dest.stat().st_size > 8000 and dest.read_bytes()[:4] == b"RIFF":
        return {"ok": True, "cached": True, "item": item, "source": None}
    last_err = "no hit"
    plain = query
    for word in (" tablet", " syrup", " capsule", " sachet", " drops", " cream", " gel", " pack"):
        plain = plain.replace(word, "")
    plain = " ".join(plain.split())
    for q in (query, plain, folder_query, name):
        try:
            hits = commons_hits(q)
        except Exception as e:
            last_err = str(e)
            time.sleep(1.5)
            continue
        for hit in hits:
            try:
                req = urllib.request.Request(hit["thumb"], headers={"User-Agent": UA})
                with urllib.request.urlopen(req, timeout=25) as resp:
                    raw = resp.read()
                if len(raw) < 4000:
                    continue
                to_webp(raw, dest)
                if dest.stat().st_size < 8000:
                    dest.unlink(missing_ok=True)
                    continue
                dest.with_suffix(".source.json").write_text(json.dumps(hit), encoding="utf-8")
                return {"ok": True, "cached": False, "item": item, "source": hit}
            except Exception as e:
                last_err = str(e)
                continue
        time.sleep(0.2)
    return {"ok": False, "item": item, "error": last_err}


def main() -> int:
    print(f"{len(ITEMS)} items", flush=True)
    if len(ITEMS) < 500:
        raise SystemExit(f"catalogue has {len(ITEMS)}, need at least 500")
    slugs = [i[1] for i in ITEMS]
    if len(slugs) != len(set(slugs)):
        raise SystemExit("duplicate slug")
    by_folder: dict[str, str] = {}
    for folder, *_rest, query in ((i[0], i[6]) for i in ITEMS):
        by_folder.setdefault(folder, query)

    results = []
    with ThreadPoolExecutor(max_workers=10) as pool:
        futs = [pool.submit(fetch_one, item, by_folder[item[0]]) for item in ITEMS]
        for n, fut in enumerate(as_completed(futs), 1):
            res = fut.result()
            results.append(res)
            name = res["item"][2]
            if res["ok"]:
                mark = "cache" if res.get("cached") else "got"
                print(f"[{n}/{len(ITEMS)}] {mark} {name}", flush=True)
            else:
                print(f"[{n}/{len(ITEMS)}] MISS {name}: {res.get('error')}", flush=True)

    ok = [r for r in results if r["ok"]]
    print(f"photos {len(ok)}/{len(ITEMS)}", flush=True)

    categories: dict[str, list] = {}
    for item in ITEMS:
        categories.setdefault(item[0], []).append(item)

    def title(folder: str) -> str:
        return " ".join(w[:1].upper() + w[1:] for w in folder.split("-"))

    tmpl_cats = []
    for folder, rows in categories.items():
        items_json = []
        for _folder, slug, name, hindi, price, unit, _q in rows:
            image = f"shelf:{folder}/{slug}.webp"
            path = SHELF / folder / f"{slug}.webp"
            if not path.is_file():
                image = ""
            entry = {"name": name, "price": price, "unit": unit, "image": image}
            if hindi:
                entry["aliases"] = [hindi]
            items_json.append(entry)
        first_img = next((i["image"] for i in items_json if i["image"]), "")
        tmpl_cats.append({
            "name": title(folder),
            "image": first_img,
            "items": items_json,
        })
    template = {
        "id": "neighbourhood",
        "title": "Neighbourhood grocery",
        "blurb": "Starter kirana list: aisles, Hindi aliases, example prices, photos. Prices are not MRP — change them for your shop.",
        "categories": tmpl_cats,
    }
    TEMPLATE_DIR.mkdir(parents=True, exist_ok=True)
    (TEMPLATE_DIR / "template.json").write_text(
        json.dumps(template, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    sources = TEMPLATE_DIR / "SOURCES.jsonl"
    with sources.open("w", encoding="utf-8") as fh:
        fh.write(json.dumps({
            "note": "Each photo is a Wikimedia Commons thumbnail re-encoded to WebP (max edge 800, quality 80). Attribute the File page.",
        }, ensure_ascii=False) + "\n")
        for res in ok:
            folder, slug, _name, _hindi, _price, _unit, query = res["item"]
            src = res.get("source") or {}
            row = {"file": f"{folder}/{slug}.webp", "search": query, "via": "Wikimedia Commons thumbnail, re-encoded to WebP"}
            if src.get("page"):
                row["commons"] = src.get("title")
                row["page"] = src["page"]
            fh.write(json.dumps(row, ensure_ascii=False) + "\n")
    missing = len(ITEMS) - len(ok)
    print(f"template written, missing photos {missing}", flush=True)
    return 0 if missing < 40 else 1


if __name__ == "__main__":
    raise SystemExit(main())
