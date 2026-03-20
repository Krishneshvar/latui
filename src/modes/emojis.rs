//! Emojis mode — searchable emoji picker for LaTUI.
//!
//! # Metadata format
//! `Item.metadata` = the raw emoji character (e.g. `"😀"`).
//!
//! # Copy backend
//! Wayland (`wl-copy`) preferred, X11 (`xclip`) fallback.

use crate::core::{item::Item, mode::Mode, searchable_item::SearchableItem};
use crate::error::LatuiError;
use crate::search::engine::SearchEngine;

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

// ── Constants ──────────────────────────────────────────────────────────────

const MAX_RECENTS: usize = 100;
const RECENT_DISPLAY_LIMIT: usize = 24;

// ── Static emoji database ──────────────────────────────────────────────────
// (glyph, name, keywords, category)

type EmojiRow = (
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static str,
);

static EMOJIS: &[EmojiRow] = &[
    // Smileys & People
    (
        "😀",
        "grinning face",
        &["happy", "smile", "grin", "joy"],
        "smileys",
    ),
    (
        "😁",
        "beaming face",
        &["happy", "grin", "teeth", "smile"],
        "smileys",
    ),
    (
        "😂",
        "face with tears of joy",
        &["lol", "laugh", "tears", "funny", "haha"],
        "smileys",
    ),
    (
        "🤣",
        "rolling on floor laughing",
        &["lol", "laugh", "rofl", "funny", "haha"],
        "smileys",
    ),
    (
        "😃",
        "grinning face with big eyes",
        &["happy", "smile", "open"],
        "smileys",
    ),
    (
        "😄",
        "grinning face with smiling eyes",
        &["happy", "smile", "laugh"],
        "smileys",
    ),
    (
        "😅",
        "grinning face with sweat",
        &["nervous", "phew", "sweat", "smile"],
        "smileys",
    ),
    (
        "😆",
        "grinning squinting face",
        &["laugh", "satisfied", "haha", "lol"],
        "smileys",
    ),
    (
        "😉",
        "winking face",
        &["wink", "joke", "playful"],
        "smileys",
    ),
    (
        "😊",
        "smiling face with smiling eyes",
        &["happy", "blush", "warm"],
        "smileys",
    ),
    (
        "😋",
        "face savoring food",
        &["yum", "delicious", "tasty", "food"],
        "smileys",
    ),
    (
        "😎",
        "smiling face with sunglasses",
        &["cool", "awesome", "sunglasses"],
        "smileys",
    ),
    (
        "😍",
        "smiling face with heart eyes",
        &["love", "adore", "crush", "heart"],
        "smileys",
    ),
    (
        "🥰",
        "smiling face with hearts",
        &["love", "affection", "warm", "hearts"],
        "smileys",
    ),
    (
        "😘",
        "face blowing a kiss",
        &["kiss", "love", "flirt", "heart"],
        "smileys",
    ),
    (
        "😗",
        "kissing face",
        &["kiss", "lips", "affection"],
        "smileys",
    ),
    (
        "🤩",
        "star struck",
        &["wow", "amazing", "stars", "excited", "celebrity"],
        "smileys",
    ),
    (
        "🥳",
        "partying face",
        &["party", "celebrate", "birthday", "festive"],
        "smileys",
    ),
    (
        "😏",
        "smirking face",
        &["smirk", "smugness", "sly", "flirt"],
        "smileys",
    ),
    (
        "😒",
        "unamused face",
        &["meh", "bored", "unimpressed", "annoyed"],
        "smileys",
    ),
    (
        "😞",
        "disappointed face",
        &["sad", "down", "unhappy", "disappointed"],
        "smileys",
    ),
    (
        "😔",
        "pensive face",
        &["sad", "thoughtful", "sorry", "melancholy"],
        "smileys",
    ),
    (
        "😟",
        "worried face",
        &["worried", "anxious", "concerned", "nervous"],
        "smileys",
    ),
    (
        "😕",
        "confused face",
        &["confused", "unsure", "hm", "puzzled"],
        "smileys",
    ),
    (
        "🙁",
        "slightly frowning face",
        &["sad", "unhappy", "frown"],
        "smileys",
    ),
    (
        "☹️",
        "frowning face",
        &["sad", "frown", "unhappy"],
        "smileys",
    ),
    (
        "😣",
        "persevering face",
        &["struggle", "pain", "suffering"],
        "smileys",
    ),
    (
        "😖",
        "confounded face",
        &["confused", "frustrated", "ugh"],
        "smileys",
    ),
    (
        "😫",
        "tired face",
        &["tired", "exhausted", "weary", "fed up"],
        "smileys",
    ),
    (
        "😩",
        "weary face",
        &["tired", "weary", "exhausted"],
        "smileys",
    ),
    (
        "🥺",
        "pleading face",
        &["plead", "beg", "cute", "puppy eyes"],
        "smileys",
    ),
    (
        "😢",
        "crying face",
        &["cry", "sad", "tear", "upset"],
        "smileys",
    ),
    (
        "😭",
        "loudly crying face",
        &["cry", "sob", "tears", "sad"],
        "smileys",
    ),
    (
        "😤",
        "face with steam from nose",
        &["triumph", "frustrated", "angry", "huff"],
        "smileys",
    ),
    (
        "😠",
        "angry face",
        &["angry", "mad", "rage", "upset"],
        "smileys",
    ),
    (
        "😡",
        "pouting face",
        &["angry", "rage", "red", "furious"],
        "smileys",
    ),
    (
        "🤬",
        "face with symbols on mouth",
        &["swear", "curse", "angry", "censored"],
        "smileys",
    ),
    (
        "🤯",
        "exploding head",
        &["mindblown", "shock", "wow", "brain"],
        "smileys",
    ),
    (
        "😳",
        "flushed face",
        &["embarrassed", "blush", "shocked"],
        "smileys",
    ),
    (
        "🥵",
        "hot face",
        &["hot", "heat", "sweat", "summer"],
        "smileys",
    ),
    (
        "🥶",
        "cold face",
        &["cold", "freeze", "winter", "ice"],
        "smileys",
    ),
    (
        "😱",
        "face screaming in fear",
        &["scream", "scared", "horror", "fear"],
        "smileys",
    ),
    (
        "😨",
        "fearful face",
        &["fear", "scared", "worried", "horror"],
        "smileys",
    ),
    (
        "😰",
        "anxious face with sweat",
        &["nervous", "sweat", "anxious", "fear"],
        "smileys",
    ),
    (
        "😓",
        "downcast face with sweat",
        &["nervous", "work", "downcast", "sweat"],
        "smileys",
    ),
    (
        "🤗",
        "smiling face with open hands",
        &["hug", "warm", "friendly", "embrace"],
        "smileys",
    ),
    (
        "🤔",
        "thinking face",
        &["think", "hmm", "ponder", "curious", "question"],
        "smileys",
    ),
    (
        "🤭",
        "face with hand over mouth",
        &["oops", "secret", "giggle", "surprised"],
        "smileys",
    ),
    (
        "🤫",
        "shushing face",
        &["quiet", "shhh", "secret", "silence"],
        "smileys",
    ),
    (
        "🤥",
        "lying face",
        &["lie", "pinocchio", "dishonest"],
        "smileys",
    ),
    (
        "😶",
        "face without mouth",
        &["silent", "mute", "quiet", "speechless"],
        "smileys",
    ),
    (
        "😐",
        "neutral face",
        &["meh", "neutral", "flat", "expressionless"],
        "smileys",
    ),
    (
        "😑",
        "expressionless face",
        &["dead", "expressionless", "blank", "bored"],
        "smileys",
    ),
    (
        "😬",
        "grimacing face",
        &["grimace", "nervous", "cringe", "awkward"],
        "smileys",
    ),
    (
        "🙄",
        "face with rolling eyes",
        &["eyeroll", "whatever", "sarcasm", "bored"],
        "smileys",
    ),
    (
        "😯",
        "hushed face",
        &["surprised", "speechless", "oh", "shock"],
        "smileys",
    ),
    (
        "😦",
        "frowning face with open mouth",
        &["shocked", "sad", "frown", "oh"],
        "smileys",
    ),
    (
        "😧",
        "anguished face",
        &["anguish", "shock", "horrified", "oh"],
        "smileys",
    ),
    (
        "😲",
        "astonished face",
        &["wow", "amazed", "shocked", "surprise"],
        "smileys",
    ),
    (
        "🥱",
        "yawning face",
        &["bored", "tired", "yawn", "sleepy"],
        "smileys",
    ),
    (
        "😴",
        "sleeping face",
        &["sleep", "zzz", "sleepy", "night", "tired"],
        "smileys",
    ),
    (
        "🤤",
        "drooling face",
        &["drool", "hungry", "want", "desire"],
        "smileys",
    ),
    (
        "😪",
        "sleepy face",
        &["sleep", "drowsy", "tired", "zzz"],
        "smileys",
    ),
    (
        "😵",
        "face with crossed-out eyes",
        &["dizzy", "dead", "fainted", "spiral"],
        "smileys",
    ),
    (
        "🤐",
        "zipper mouth face",
        &["secret", "silent", "zip", "mouth"],
        "smileys",
    ),
    (
        "🥴",
        "woozy face",
        &["drunk", "dizzy", "woozy", "unwell"],
        "smileys",
    ),
    (
        "🤢",
        "nauseated face",
        &["sick", "nausea", "gross", "yuck", "vomit"],
        "smileys",
    ),
    (
        "🤮",
        "face vomiting",
        &["sick", "vomit", "gross", "ill", "yuck"],
        "smileys",
    ),
    (
        "🤧",
        "sneezing face",
        &["sick", "sneeze", "ill", "cold", "tissue"],
        "smileys",
    ),
    (
        "😷",
        "face with medical mask",
        &["sick", "ill", "mask", "covid", "doctor"],
        "smileys",
    ),
    (
        "🤒",
        "face with thermometer",
        &["sick", "ill", "fever", "temperature", "cold"],
        "smileys",
    ),
    (
        "🤕",
        "face with head bandage",
        &["hurt", "injured", "bandage", "ouch", "pain"],
        "smileys",
    ),
    (
        "🤑",
        "money mouth face",
        &["money", "rich", "dollar", "greedy"],
        "smileys",
    ),
    (
        "😈",
        "smiling face with horns",
        &["devil", "evil", "mischief", "horns", "imp"],
        "smileys",
    ),
    (
        "👿",
        "angry face with horns",
        &["devil", "evil", "demon", "angry", "cursed"],
        "smileys",
    ),
    (
        "💀",
        "skull",
        &["death", "dead", "skull", "danger", "danger"],
        "smileys",
    ),
    (
        "☠️",
        "skull and crossbones",
        &["danger", "poison", "death", "pirate"],
        "smileys",
    ),
    (
        "💩",
        "pile of poo",
        &["poop", "crap", "shit", "funny"],
        "smileys",
    ),
    (
        "🤡",
        "clown face",
        &["clown", "joker", "scary", "circus"],
        "smileys",
    ),
    (
        "👹",
        "ogre",
        &["monster", "devil", "creature", "scary"],
        "smileys",
    ),
    (
        "👺",
        "goblin",
        &["monster", "creature", "scary", "japan"],
        "smileys",
    ),
    (
        "👻",
        "ghost",
        &["ghost", "spooky", "halloween", "boo", "spirit"],
        "smileys",
    ),
    (
        "👽",
        "alien",
        &["alien", "ufo", "space", "extraterrestrial"],
        "smileys",
    ),
    (
        "👾",
        "alien monster",
        &["alien", "game", "monster", "pixel", "video game"],
        "smileys",
    ),
    (
        "🤖",
        "robot",
        &["robot", "bot", "ai", "machine", "tech"],
        "smileys",
    ),
    // Gestures & Body
    (
        "👋",
        "waving hand",
        &["wave", "hello", "hi", "bye", "goodbye"],
        "people",
    ),
    (
        "🤚",
        "raised back of hand",
        &["hand", "raise", "stop"],
        "people",
    ),
    (
        "✋",
        "raised hand",
        &["stop", "high five", "hand", "halt"],
        "people",
    ),
    (
        "🖖",
        "vulcan salute",
        &["spock", "star trek", "vulcan", "live long"],
        "people",
    ),
    (
        "👌",
        "ok hand",
        &["ok", "perfect", "agree", "fine", "good"],
        "people",
    ),
    (
        "🤌",
        "pinched fingers",
        &["italian", "gesture", "perfect", "chef kiss"],
        "people",
    ),
    (
        "✌️",
        "victory hand",
        &["peace", "victory", "v", "two", "win"],
        "people",
    ),
    (
        "🤞",
        "crossed fingers",
        &["luck", "hope", "fingers crossed", "wish"],
        "people",
    ),
    (
        "🤟",
        "love you gesture",
        &["love", "rock", "ily", "sign language"],
        "people",
    ),
    (
        "🤘",
        "sign of the horns",
        &["rock", "metal", "horns", "rock on"],
        "people",
    ),
    (
        "👊",
        "oncoming fist",
        &["punch", "fist", "fight", "power"],
        "people",
    ),
    (
        "✊",
        "raised fist",
        &["power", "fist", "solidarity", "resist"],
        "people",
    ),
    (
        "🤛",
        "left facing fist",
        &["fist bump", "left", "power"],
        "people",
    ),
    (
        "🤜",
        "right facing fist",
        &["fist bump", "right", "power"],
        "people",
    ),
    (
        "👏",
        "clapping hands",
        &["clap", "applause", "bravo", "congrats"],
        "people",
    ),
    (
        "🙌",
        "raising hands",
        &["celebrate", "hooray", "praise", "party"],
        "people",
    ),
    (
        "🤲",
        "palms up",
        &["help", "prayer", "ask", "give"],
        "people",
    ),
    (
        "🤝",
        "handshake",
        &["deal", "agreement", "shake", "partnership"],
        "people",
    ),
    (
        "🙏",
        "folded hands",
        &["pray", "please", "thanks", "namaste", "hope"],
        "people",
    ),
    (
        "💪",
        "flexed biceps",
        &["strong", "muscle", "power", "gym", "flex"],
        "people",
    ),
    (
        "👍",
        "thumbs up",
        &["good", "yes", "approve", "like", "ok", "cool"],
        "people",
    ),
    (
        "👎",
        "thumbs down",
        &["bad", "no", "dislike", "disapprove"],
        "people",
    ),
    (
        "☝️",
        "index pointing up",
        &["point", "up", "one", "attention"],
        "people",
    ),
    (
        "👆",
        "backhand index pointing up",
        &["up", "point", "above"],
        "people",
    ),
    (
        "👇",
        "backhand index pointing down",
        &["down", "point", "below"],
        "people",
    ),
    (
        "👈",
        "backhand index pointing left",
        &["left", "point", "direction"],
        "people",
    ),
    (
        "👉",
        "backhand index pointing right",
        &["right", "point", "direction", "click"],
        "people",
    ),
    (
        "🖕",
        "middle finger",
        &["rude", "offensive", "gesture", "finger"],
        "people",
    ),
    (
        "🖐️",
        "hand with fingers splayed",
        &["hand", "five", "spread", "stop"],
        "people",
    ),
    (
        "👋",
        "waving hand",
        &["wave", "bye", "hello", "hi"],
        "people",
    ),
    (
        "🤙",
        "call me hand",
        &["call", "phone", "hang loose", "shaka"],
        "people",
    ),
    // Hearts
    (
        "❤️",
        "red heart",
        &["love", "heart", "red", "romance", "favorite"],
        "symbols",
    ),
    (
        "🧡",
        "orange heart",
        &["love", "heart", "orange", "warm"],
        "symbols",
    ),
    (
        "💛",
        "yellow heart",
        &["love", "heart", "yellow", "friendship"],
        "symbols",
    ),
    (
        "💚",
        "green heart",
        &["love", "heart", "green", "nature"],
        "symbols",
    ),
    (
        "💙",
        "blue heart",
        &["love", "heart", "blue", "trust"],
        "symbols",
    ),
    (
        "💜",
        "purple heart",
        &["love", "heart", "purple", "compassion"],
        "symbols",
    ),
    (
        "🖤",
        "black heart",
        &["love", "heart", "dark", "goth"],
        "symbols",
    ),
    (
        "🤍",
        "white heart",
        &["love", "heart", "white", "pure"],
        "symbols",
    ),
    (
        "🤎",
        "brown heart",
        &["love", "heart", "brown", "earth"],
        "symbols",
    ),
    (
        "💔",
        "broken heart",
        &["heartbreak", "sad", "love", "loss"],
        "symbols",
    ),
    (
        "❣️",
        "heart exclamation",
        &["love", "heart", "mark", "emphasis"],
        "symbols",
    ),
    (
        "💕",
        "two hearts",
        &["love", "hearts", "romance", "affection"],
        "symbols",
    ),
    (
        "💞",
        "revolving hearts",
        &["love", "hearts", "rotating", "affection"],
        "symbols",
    ),
    (
        "💓",
        "beating heart",
        &["love", "heart", "pulse", "beating"],
        "symbols",
    ),
    (
        "💗",
        "growing heart",
        &["love", "heart", "growing", "excited"],
        "symbols",
    ),
    (
        "💖",
        "sparkling heart",
        &["love", "heart", "sparkle", "excited"],
        "symbols",
    ),
    (
        "💘",
        "heart with arrow",
        &["love", "cupid", "arrow", "heart", "romantic"],
        "symbols",
    ),
    (
        "💝",
        "heart with ribbon",
        &["love", "gift", "heart", "ribbon", "present"],
        "symbols",
    ),
    (
        "💟",
        "heart decoration",
        &["love", "heart", "purple", "decoration"],
        "symbols",
    ),
    // Nature & Animals
    (
        "🐶",
        "dog face",
        &["dog", "puppy", "pet", "animal", "woof"],
        "animals",
    ),
    (
        "🐱",
        "cat face",
        &["cat", "kitten", "pet", "animal", "meow"],
        "animals",
    ),
    (
        "🐭",
        "mouse face",
        &["mouse", "animal", "rodent", "pet"],
        "animals",
    ),
    (
        "🐹",
        "hamster",
        &["hamster", "pet", "animal", "cute"],
        "animals",
    ),
    (
        "🐰",
        "rabbit face",
        &["rabbit", "bunny", "easter", "animal", "pet"],
        "animals",
    ),
    (
        "🦊",
        "fox",
        &["fox", "animal", "clever", "sly", "cute"],
        "animals",
    ),
    (
        "🐻",
        "bear",
        &["bear", "animal", "teddy", "cute", "forest"],
        "animals",
    ),
    (
        "🐼",
        "panda",
        &["panda", "animal", "cute", "china", "bamboo"],
        "animals",
    ),
    (
        "🐨",
        "koala",
        &["koala", "animal", "australia", "cute", "bear"],
        "animals",
    ),
    (
        "🐯",
        "tiger face",
        &["tiger", "animal", "fierce", "jungle", "stripe"],
        "animals",
    ),
    (
        "🦁",
        "lion",
        &["lion", "animal", "king", "safari", "fierce"],
        "animals",
    ),
    (
        "🐮",
        "cow face",
        &["cow", "animal", "farm", "moo", "milk"],
        "animals",
    ),
    (
        "🐷",
        "pig face",
        &["pig", "animal", "farm", "oink", "bacon"],
        "animals",
    ),
    (
        "🐸",
        "frog",
        &["frog", "animal", "green", "pond", "leap"],
        "animals",
    ),
    (
        "🐵",
        "monkey face",
        &["monkey", "animal", "primate", "funny", "ape"],
        "animals",
    ),
    (
        "🐔",
        "chicken",
        &["chicken", "bird", "animal", "farm", "cluck"],
        "animals",
    ),
    (
        "🐧",
        "penguin",
        &["penguin", "bird", "animal", "arctic", "cute"],
        "animals",
    ),
    (
        "🐦",
        "bird",
        &["bird", "animal", "fly", "tweet", "nature"],
        "animals",
    ),
    (
        "🦆",
        "duck",
        &["duck", "bird", "animal", "quack", "water"],
        "animals",
    ),
    (
        "🦅",
        "eagle",
        &["eagle", "bird", "animal", "freedom", "majestic"],
        "animals",
    ),
    (
        "🦋",
        "butterfly",
        &["butterfly", "insect", "nature", "pretty", "fly"],
        "animals",
    ),
    (
        "🐛",
        "bug",
        &["bug", "insect", "caterpillar", "nature"],
        "animals",
    ),
    (
        "🐝",
        "honeybee",
        &["bee", "insect", "honey", "sting", "nature"],
        "animals",
    ),
    (
        "🌸",
        "cherry blossom",
        &["flower", "sakura", "japan", "spring", "pink"],
        "nature",
    ),
    (
        "🌺",
        "hibiscus",
        &["flower", "tropical", "pink", "nature", "hawaii"],
        "nature",
    ),
    (
        "🌻",
        "sunflower",
        &["sunflower", "flower", "sun", "yellow", "nature"],
        "nature",
    ),
    (
        "🌹",
        "rose",
        &["rose", "flower", "love", "romance", "red"],
        "nature",
    ),
    (
        "🌷",
        "tulip",
        &["tulip", "flower", "spring", "nature", "pink"],
        "nature",
    ),
    (
        "🌱",
        "seedling",
        &["plant", "grow", "sprout", "nature", "green"],
        "nature",
    ),
    (
        "🌲",
        "evergreen tree",
        &["tree", "nature", "green", "forest", "pine"],
        "nature",
    ),
    (
        "🌳",
        "deciduous tree",
        &["tree", "nature", "green", "forest", "oak"],
        "nature",
    ),
    (
        "🍀",
        "four leaf clover",
        &["luck", "clover", "green", "lucky", "irish"],
        "nature",
    ),
    (
        "🍁",
        "maple leaf",
        &["leaf", "autumn", "fall", "canada", "red"],
        "nature",
    ),
    (
        "🌊",
        "water wave",
        &["wave", "ocean", "sea", "water", "surf"],
        "nature",
    ),
    (
        "⭐",
        "star",
        &["star", "favorite", "night", "awesome", "rating"],
        "nature",
    ),
    (
        "🌟",
        "glowing star",
        &["star", "shine", "glow", "awesome", "favorite"],
        "nature",
    ),
    (
        "🌈",
        "rainbow",
        &["rainbow", "color", "rain", "pride", "beautiful"],
        "nature",
    ),
    (
        "☀️",
        "sun",
        &["sun", "sunny", "warm", "summer", "weather"],
        "nature",
    ),
    (
        "🌙",
        "crescent moon",
        &["moon", "night", "sleep", "crescent", "night"],
        "nature",
    ),
    (
        "⚡",
        "lightning",
        &["lightning", "electric", "thunder", "storm", "fast", "bolt"],
        "nature",
    ),
    (
        "🔥",
        "fire",
        &["fire", "hot", "flame", "lit", "burn", "spicy"],
        "nature",
    ),
    (
        "💧",
        "droplet",
        &["water", "drop", "blue", "liquid", "tear"],
        "nature",
    ),
    (
        "❄️",
        "snowflake",
        &["snow", "cold", "winter", "ice", "freeze", "flake"],
        "nature",
    ),
    (
        "🌍",
        "globe europe africa",
        &["world", "earth", "globe", "global", "planet"],
        "nature",
    ),
    (
        "🌎",
        "globe americas",
        &["world", "earth", "globe", "global", "map"],
        "nature",
    ),
    (
        "🌏",
        "globe asia australia",
        &["world", "earth", "globe", "global", "asia"],
        "nature",
    ),
    // Food & Drink
    (
        "🍎",
        "red apple",
        &["apple", "fruit", "food", "red", "healthy"],
        "food",
    ),
    (
        "🍊",
        "tangerine",
        &["orange", "fruit", "food", "citrus", "mandarin"],
        "food",
    ),
    (
        "🍋",
        "lemon",
        &["lemon", "fruit", "food", "citrus", "sour", "yellow"],
        "food",
    ),
    (
        "🍇",
        "grapes",
        &["grapes", "fruit", "food", "purple", "wine"],
        "food",
    ),
    (
        "🍓",
        "strawberry",
        &["strawberry", "fruit", "food", "red", "sweet"],
        "food",
    ),
    (
        "🍔",
        "hamburger",
        &["burger", "food", "fast food", "meat", "lunch"],
        "food",
    ),
    (
        "🍕",
        "pizza",
        &["pizza", "food", "slice", "italian", "cheese"],
        "food",
    ),
    (
        "🍣",
        "sushi",
        &["sushi", "food", "japanese", "fish", "rice"],
        "food",
    ),
    (
        "🍜",
        "steaming bowl",
        &["noodle", "ramen", "food", "soup", "japanese"],
        "food",
    ),
    (
        "🍦",
        "soft ice cream",
        &["ice cream", "dessert", "sweet", "cone", "cold"],
        "food",
    ),
    (
        "🎂",
        "birthday cake",
        &["cake", "birthday", "celebrate", "candles", "sweet"],
        "food",
    ),
    (
        "☕",
        "hot beverage",
        &["coffee", "tea", "hot", "drink", "cafe", "morning"],
        "food",
    ),
    (
        "🍺",
        "beer mug",
        &["beer", "drink", "alcohol", "cheers", "pub"],
        "food",
    ),
    (
        "🍷",
        "wine glass",
        &["wine", "drink", "alcohol", "red", "cheers"],
        "food",
    ),
    (
        "🥂",
        "clinking glasses",
        &["cheers", "toast", "celebrate", "drink", "party"],
        "food",
    ),
    (
        "🥤",
        "cup with straw",
        &["drink", "soda", "juice", "cup", "beverage"],
        "food",
    ),
    // Activities & Sports
    (
        "⚽",
        "soccer ball",
        &["soccer", "football", "sport", "ball", "game", "kick"],
        "activities",
    ),
    (
        "🏀",
        "basketball",
        &["basketball", "sport", "ball", "game", "nba", "hoop"],
        "activities",
    ),
    (
        "🏈",
        "american football",
        &["football", "sport", "nfl", "ball", "game"],
        "activities",
    ),
    (
        "⚾",
        "baseball",
        &["baseball", "sport", "ball", "mlb", "game", "pitch"],
        "activities",
    ),
    (
        "🎾",
        "tennis",
        &["tennis", "sport", "ball", "court", "racket"],
        "activities",
    ),
    (
        "🏐",
        "volleyball",
        &["volleyball", "sport", "ball", "beach", "game"],
        "activities",
    ),
    (
        "🎮",
        "video game",
        &["gaming", "controller", "game", "play", "videogame"],
        "activities",
    ),
    (
        "🎲",
        "game die",
        &["dice", "game", "random", "roll", "chance", "board game"],
        "activities",
    ),
    (
        "🎯",
        "direct hit",
        &["target", "bullseye", "aim", "goal", "darts"],
        "activities",
    ),
    (
        "🏆",
        "trophy",
        &["trophy", "win", "award", "champion", "prize", "victory"],
        "activities",
    ),
    (
        "🥇",
        "1st place medal",
        &["gold", "medal", "first", "win", "champion"],
        "activities",
    ),
    (
        "🎵",
        "musical note",
        &["music", "note", "song", "audio", "melody"],
        "activities",
    ),
    (
        "🎶",
        "musical notes",
        &["music", "notes", "song", "audio", "melody"],
        "activities",
    ),
    (
        "🎸",
        "guitar",
        &["guitar", "music", "rock", "instrument", "band"],
        "activities",
    ),
    (
        "🎹",
        "musical keyboard",
        &["piano", "music", "keyboard", "instrument", "keys"],
        "activities",
    ),
    (
        "🎤",
        "microphone",
        &["mic", "sing", "karaoke", "music", "perform"],
        "activities",
    ),
    (
        "🎬",
        "clapper board",
        &["film", "movie", "cinema", "action", "director"],
        "activities",
    ),
    (
        "🎭",
        "performing arts",
        &["theater", "arts", "drama", "comedy", "tragedy"],
        "activities",
    ),
    (
        "📚",
        "books",
        &["book", "read", "study", "library", "education", "learn"],
        "activities",
    ),
    (
        "🎨",
        "artist palette",
        &["art", "paint", "creative", "color", "design"],
        "activities",
    ),
    // Objects & Tech
    (
        "💻",
        "laptop",
        &["laptop", "computer", "tech", "coding", "work", "screen"],
        "objects",
    ),
    (
        "🖥️",
        "desktop computer",
        &["desktop", "computer", "monitor", "tech", "work"],
        "objects",
    ),
    (
        "📱",
        "mobile phone",
        &["phone", "mobile", "smartphone", "call", "text"],
        "objects",
    ),
    (
        "⌨️",
        "keyboard",
        &["keyboard", "typing", "computer", "input", "tech"],
        "objects",
    ),
    (
        "🖱️",
        "computer mouse",
        &["mouse", "pointer", "click", "computer", "tech"],
        "objects",
    ),
    (
        "🖨️",
        "printer",
        &["printer", "print", "paper", "office", "document"],
        "objects",
    ),
    (
        "💾",
        "floppy disk",
        &["save", "disk", "storage", "data", "old", "backup"],
        "objects",
    ),
    (
        "💿",
        "optical disk",
        &["cd", "disc", "music", "data", "storage"],
        "objects",
    ),
    (
        "📷",
        "camera",
        &["camera", "photo", "picture", "shoot", "photography"],
        "objects",
    ),
    (
        "📸",
        "camera with flash",
        &["camera", "photo", "selfie", "flash", "picture"],
        "objects",
    ),
    (
        "📞",
        "telephone receiver",
        &["phone", "call", "telephone", "contact"],
        "objects",
    ),
    (
        "📡",
        "satellite antenna",
        &["satellite", "signal", "wifi", "antenna", "broadcast"],
        "objects",
    ),
    (
        "🔋",
        "battery",
        &["battery", "power", "energy", "charge", "electric"],
        "objects",
    ),
    (
        "🔌",
        "electric plug",
        &["plug", "power", "electric", "charge", "cable"],
        "objects",
    ),
    (
        "💡",
        "light bulb",
        &["idea", "light", "bright", "bulb", "invention", "tip"],
        "objects",
    ),
    (
        "🔦",
        "flashlight",
        &["flashlight", "light", "dark", "torch"],
        "objects",
    ),
    (
        "🔒",
        "locked",
        &["lock", "secure", "private", "security", "closed"],
        "objects",
    ),
    (
        "🔓",
        "unlocked",
        &["unlock", "open", "access", "security"],
        "objects",
    ),
    (
        "🔑",
        "key",
        &["key", "lock", "security", "access", "door"],
        "objects",
    ),
    (
        "🔨",
        "hammer",
        &["hammer", "tool", "build", "fix", "construction"],
        "objects",
    ),
    (
        "🪛",
        "screwdriver",
        &["screwdriver", "tool", "fix", "repair", "build"],
        "objects",
    ),
    (
        "🔧",
        "wrench",
        &["wrench", "tool", "fix", "repair", "mechanic"],
        "objects",
    ),
    (
        "⚙️",
        "gear",
        &["gear", "settings", "config", "mechanic", "cog", "settings"],
        "objects",
    ),
    (
        "🧲",
        "magnet",
        &["magnet", "attract", "metal", "force", "pull"],
        "objects",
    ),
    (
        "💊",
        "pill",
        &["pill", "medicine", "drug", "health", "doctor"],
        "objects",
    ),
    (
        "🧪",
        "test tube",
        &["science", "experiment", "chemistry", "lab", "test"],
        "objects",
    ),
    (
        "🔬",
        "microscope",
        &["microscope", "science", "biology", "lab", "zoom"],
        "objects",
    ),
    (
        "🔭",
        "telescope",
        &["telescope", "space", "stars", "astronomy", "observe"],
        "objects",
    ),
    (
        "📊",
        "bar chart",
        &["chart", "graph", "data", "stats", "analytics"],
        "objects",
    ),
    (
        "📈",
        "chart increasing",
        &["chart", "up", "growth", "profit", "trend", "increase"],
        "objects",
    ),
    (
        "📉",
        "chart decreasing",
        &["chart", "down", "loss", "decrease", "trend", "drop"],
        "objects",
    ),
    (
        "📝",
        "memo",
        &["note", "write", "memo", "document", "pencil", "edit"],
        "objects",
    ),
    (
        "📌",
        "pushpin",
        &["pin", "location", "mark", "note", "important"],
        "objects",
    ),
    (
        "🗑️",
        "wastebasket",
        &["trash", "delete", "remove", "garbage", "recycle"],
        "objects",
    ),
    (
        "📦",
        "package",
        &["box", "package", "ship", "delivery", "parcel"],
        "objects",
    ),
    (
        "📬",
        "open mailbox with raised flag",
        &["mail", "email", "inbox", "message", "post"],
        "objects",
    ),
    (
        "🗓️",
        "spiral calendar",
        &["calendar", "date", "schedule", "event", "plan"],
        "objects",
    ),
    (
        "⏰",
        "alarm clock",
        &["alarm", "clock", "time", "wake", "morning"],
        "objects",
    ),
    (
        "⌚",
        "watch",
        &["watch", "time", "clock", "wrist", "schedule"],
        "objects",
    ),
    // Symbols & Signs
    (
        "✅",
        "check mark button",
        &["check", "done", "correct", "yes", "tick", "complete"],
        "symbols",
    ),
    (
        "❌",
        "cross mark",
        &["no", "wrong", "error", "cancel", "false", "x", "delete"],
        "symbols",
    ),
    (
        "⚠️",
        "warning",
        &["warning", "caution", "alert", "danger", "careful"],
        "symbols",
    ),
    (
        "🚫",
        "prohibited",
        &["no", "ban", "block", "forbidden", "stop", "not allowed"],
        "symbols",
    ),
    (
        "💯",
        "hundred points",
        &["100", "perfect", "score", "full", "yes", "fire"],
        "symbols",
    ),
    (
        "🔞",
        "no one under eighteen",
        &["adult", "18", "mature", "nsfw", "age"],
        "symbols",
    ),
    (
        "✨",
        "sparkles",
        &["sparkle", "star", "magic", "shine", "glitter", "special"],
        "symbols",
    ),
    (
        "🎉",
        "party popper",
        &[
            "party",
            "celebrate",
            "congrats",
            "tada",
            "festive",
            "hooray",
        ],
        "symbols",
    ),
    (
        "🎊",
        "confetti ball",
        &["celebrate", "confetti", "party", "congrats", "festive"],
        "symbols",
    ),
    (
        "🎁",
        "wrapped gift",
        &["gift", "present", "birthday", "surprise", "box"],
        "symbols",
    ),
    (
        "🏷️",
        "label",
        &["label", "tag", "price", "name", "mark"],
        "symbols",
    ),
    (
        "💬",
        "speech balloon",
        &[
            "chat",
            "message",
            "talk",
            "comment",
            "bubble",
            "conversation",
        ],
        "symbols",
    ),
    (
        "💭",
        "thought balloon",
        &["thought", "think", "wonder", "pondering"],
        "symbols",
    ),
    (
        "📢",
        "loudspeaker",
        &[
            "announce",
            "loud",
            "speaker",
            "broadcast",
            "alert",
            "megaphone",
        ],
        "symbols",
    ),
    (
        "🔔",
        "bell",
        &["notification", "bell", "alert", "ring", "ding"],
        "symbols",
    ),
    (
        "🔕",
        "bell with slash",
        &["silent", "mute", "no notification", "quiet"],
        "symbols",
    ),
    (
        "❓",
        "red question mark",
        &["question", "what", "huh", "help", "ask", "query"],
        "symbols",
    ),
    (
        "❗",
        "red exclamation mark",
        &["exclamation", "important", "alert", "warning", "!"],
        "symbols",
    ),
    (
        "💤",
        "zzz",
        &["sleep", "tired", "zzz", "sleepy", "snooze", "rest"],
        "symbols",
    ),
    (
        "🆕",
        "new button",
        &["new", "fresh", "badge", "label"],
        "symbols",
    ),
    (
        "🆓",
        "free button",
        &["free", "gratis", "no cost", "badge"],
        "symbols",
    ),
    (
        "🔝",
        "top arrow",
        &["top", "up", "above", "back to top"],
        "symbols",
    ),
    (
        "🆙",
        "up button",
        &["up", "above", "badge", "label"],
        "symbols",
    ),
    (
        "▶️",
        "play button",
        &["play", "video", "start", "media"],
        "symbols",
    ),
    (
        "⏸️",
        "pause button",
        &["pause", "stop", "media", "break"],
        "symbols",
    ),
    (
        "⏹️",
        "stop button",
        &["stop", "media", "end", "square"],
        "symbols",
    ),
    (
        "🔀",
        "shuffle",
        &["shuffle", "random", "mix", "music"],
        "symbols",
    ),
    (
        "🔁",
        "repeat button",
        &["repeat", "loop", "again", "cycle"],
        "symbols",
    ),
    // Travel & Places
    (
        "🚗",
        "automobile",
        &["car", "drive", "vehicle", "auto", "road"],
        "travel",
    ),
    (
        "🚕",
        "taxi",
        &["taxi", "cab", "drive", "yellow", "transport"],
        "travel",
    ),
    (
        "🚙",
        "sport utility vehicle",
        &["suv", "car", "vehicle", "drive"],
        "travel",
    ),
    (
        "🚌",
        "bus",
        &["bus", "transport", "public", "vehicle", "commute"],
        "travel",
    ),
    (
        "🚂",
        "locomotive",
        &["train", "railway", "transport", "steam"],
        "travel",
    ),
    (
        "✈️",
        "airplane",
        &["plane", "fly", "travel", "flight", "airport"],
        "travel",
    ),
    (
        "🚀",
        "rocket",
        &["rocket", "space", "launch", "nasa", "fast"],
        "travel",
    ),
    (
        "🛸",
        "flying saucer",
        &["ufo", "alien", "space", "sci-fi"],
        "travel",
    ),
    (
        "🛳️",
        "passenger ship",
        &["ship", "cruise", "ocean", "travel", "sea"],
        "travel",
    ),
    (
        "⛵",
        "sailboat",
        &["sail", "boat", "sea", "ocean", "travel"],
        "travel",
    ),
    (
        "🏠",
        "house",
        &["home", "house", "building", "live", "shelter"],
        "travel",
    ),
    (
        "🏢",
        "office building",
        &["office", "work", "building", "city", "business"],
        "travel",
    ),
    (
        "🏥",
        "hospital",
        &["hospital", "health", "medical", "doctor", "nurse"],
        "travel",
    ),
    (
        "🏫",
        "school",
        &["school", "education", "learn", "study", "building"],
        "travel",
    ),
    (
        "🌆",
        "cityscape at dusk",
        &["city", "dusk", "downtown", "urban", "skyline"],
        "travel",
    ),
    (
        "🗺️",
        "world map",
        &["map", "world", "navigate", "travel", "geography"],
        "travel",
    ),
    (
        "🏔️",
        "snow capped mountain",
        &["mountain", "snow", "nature", "climb", "peak"],
        "travel",
    ),
    (
        "🏖️",
        "beach with umbrella",
        &["beach", "summer", "vacation", "sun", "sand", "sea"],
        "travel",
    ),
    (
        "🌴",
        "palm tree",
        &["palm", "tropical", "beach", "summer", "island", "tree"],
        "travel",
    ),
];

// ── Recent entry ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecentEmoji {
    glyph: String,
    use_count: u32,
    last_used: u64,
}

// ── Copy backend ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum CopyBackend {
    Wayland,
    X11,
    None,
}

impl CopyBackend {
    fn detect() -> Self {
        let wayland = std::env::var("WAYLAND_DISPLAY")
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        if wayland && cmd_exists("wl-copy") {
            return CopyBackend::Wayland;
        }
        let display = std::env::var("DISPLAY")
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        if display && cmd_exists("xclip") {
            return CopyBackend::X11;
        }
        CopyBackend::None
    }
}

fn cmd_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ── EmojisMode ─────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct EmojisMode {
    /// Searchable index over the static database.
    searchable: Vec<SearchableItem>,
    /// Recently / frequently used emojis (most recent first).
    recents: VecDeque<RecentEmoji>,
    search_engine: SearchEngine,
    backend: CopyBackend,
    recents_path: Option<PathBuf>,
    last_action_time: Option<Instant>,
    dirty: bool,
}

impl EmojisMode {
    pub fn new() -> Self {
        Self {
            searchable: Vec::new(),
            recents: VecDeque::new(),
            search_engine: SearchEngine::new(),
            backend: CopyBackend::detect(),
            recents_path: None,
            last_action_time: None,
            dirty: false,
        }
    }

    // ── Persistence ─────────────────────────────────────────────────────

    fn load_recents(&mut self) -> Result<(), LatuiError> {
        use xdg::BaseDirectories;
        let xdg = BaseDirectories::with_prefix("latui");
        let path = xdg
            .place_data_file("emoji_recents.json")
            .map_err(|e| LatuiError::Io(std::io::Error::other(e)))?;

        self.recents_path = Some(path.clone());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
                let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
            }
        }

        if !path.exists() {
            return Ok(());
        }

        if let Ok(meta) = std::fs::metadata(&path)
            && meta.len() > 512 * 1024
        {
            return Ok(());
        }

        if let Ok(data) = std::fs::read_to_string(&path)
            && let Ok(mut entries) = serde_json::from_str::<Vec<RecentEmoji>>(&data)
        {
            entries.truncate(MAX_RECENTS);
            self.recents = entries.into();
            tracing::info!("Loaded {} emoji recents", self.recents.len());
        }
        Ok(())
    }

    fn save_recents(&mut self) -> Result<(), LatuiError> {
        if !self.dirty {
            return Ok(());
        }
        let path = match &self.recents_path {
            Some(p) => p.clone(),
            None => return Ok(()),
        };
        let entries: Vec<RecentEmoji> = self.recents.iter().cloned().collect();
        let mut tmp_path = path.clone();
        tmp_path.set_extension("tmp");

        let file = std::fs::File::create(&tmp_path)?;
        let writer = std::io::BufWriter::new(file);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600));
        }

        serde_json::to_writer_pretty(writer, &entries)
            .map_err(|e| LatuiError::Io(std::io::Error::other(e)))?;
        std::fs::rename(&tmp_path, &path)?;
        self.dirty = false;
        Ok(())
    }

    // ── History ─────────────────────────────────────────────────────────

    fn record_use(&mut self, glyph: &str) {
        let now = current_timestamp();
        if let Some(pos) = self.recents.iter().position(|e| e.glyph == glyph) {
            let mut entry = self.recents.remove(pos).unwrap();
            entry.use_count += 1;
            entry.last_used = now;
            self.recents.push_front(entry);
        } else {
            self.recents.push_front(RecentEmoji {
                glyph: glyph.to_string(),
                use_count: 1,
                last_used: now,
            });
            if self.recents.len() > MAX_RECENTS {
                self.recents.pop_back();
            }
        }
        self.dirty = true;
    }

    // ── Index building ───────────────────────────────────────────────────

    fn build_index(&mut self) {
        self.searchable = EMOJIS
            .iter()
            .map(|(glyph, name, keywords, category)| {
                let title = format!("{} {}", glyph, name);
                let all_keywords = keywords.join(" ");

                let item = Item {
                    id: format!("emoji:{}", glyph),
                    title,
                    search_text: name.to_lowercase(),
                    description: Some(format!("{} · {}", category, all_keywords)),
                    icon: None,
                    metadata: Some(glyph.to_string()),
                };

                let mut si = SearchableItem::new(item)
                    .with_field("name", name, 10.0)
                    .with_field("category", category, 4.0);

                for kw in *keywords {
                    si = si.with_field("keyword", kw, 8.0);
                }
                si
            })
            .collect();

        tracing::info!("Emoji index built: {} entries", self.searchable.len());
    }

    // ── Recent display ───────────────────────────────────────────────────

    fn get_recent_display(&self) -> Vec<Item> {
        let limit = self.recents.len().min(RECENT_DISPLAY_LIMIT);
        let mut scored: Vec<(Item, f64)> = self
            .recents
            .iter()
            .take(limit)
            .enumerate()
            .filter_map(|(idx, entry)| {
                // Find the emoji name from static data
                let row = EMOJIS
                    .iter()
                    .find(|(g, _, _, _)| *g == entry.glyph.as_str())?;
                let (glyph, name, keywords, category) = row;
                let all_kw = keywords.join(" ");
                let item = Item {
                    id: format!("emoji:{}", glyph),
                    title: format!("{} {}", glyph, name),
                    search_text: name.to_lowercase(),
                    description: Some(format!("{} · {}", category, all_kw)),
                    icon: None,
                    metadata: Some(glyph.to_string()),
                };
                let score =
                    (limit - idx) as f64 * 10.0 + (entry.use_count as f64 + 1.0).ln() * 15.0;
                Some((item, score))
            })
            .collect();

        // Fall back to first rows from static database if recents is empty
        if scored.is_empty() {
            return EMOJIS
                .iter()
                .take(RECENT_DISPLAY_LIMIT)
                .map(|(glyph, name, keywords, category)| Item {
                    id: format!("emoji:{}", glyph),
                    title: format!("{} {}", glyph, name),
                    search_text: name.to_lowercase(),
                    description: Some(format!("{} · {}", keywords.join(" "), category)),
                    icon: None,
                    metadata: Some(glyph.to_string()),
                })
                .collect();
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().map(|(item, _)| item).collect()
    }

    // ── Clipboard write ──────────────────────────────────────────────────

    fn copy_to_clipboard(&self, text: &str) -> Result<(), LatuiError> {
        match &self.backend {
            CopyBackend::Wayland => {
                let mut child = Command::new("wl-copy")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .map_err(LatuiError::Io)?;
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    stdin.write_all(text.as_bytes()).map_err(LatuiError::Io)?;
                }
                child.wait().map_err(LatuiError::Io)?;
            }
            CopyBackend::X11 => {
                let mut child = Command::new("xclip")
                    .args(["-selection", "clipboard"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .map_err(LatuiError::Io)?;
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    stdin.write_all(text.as_bytes()).map_err(LatuiError::Io)?;
                }
                child.wait().map_err(LatuiError::Io)?;
            }
            CopyBackend::None => {
                return Err(LatuiError::App(
                    "No clipboard tool found (install wl-copy or xclip)".to_string(),
                ));
            }
        }
        tracing::debug!("Copied emoji to clipboard: {}", text);
        Ok(())
    }
}

impl Default for EmojisMode {
    fn default() -> Self {
        Self::new()
    }
}

// ── Mode impl ──────────────────────────────────────────────────────────────

impl Mode for EmojisMode {
    fn name(&self) -> &str {
        "emojis"
    }
    fn icon(&self) -> &str {
        "😀"
    }
    fn description(&self) -> &str {
        "Emoji Picker"
    }

    fn stays_open(&self) -> bool {
        true
    }

    fn load(&mut self) -> Result<(), LatuiError> {
        tracing::debug!("Loading emojis mode");
        self.build_index();
        self.load_recents()?;
        tracing::info!("Emojis mode ready ({} emojis)", self.searchable.len());
        Ok(())
    }

    fn search(&mut self, query: &str) -> Vec<Item> {
        let start = Instant::now();

        if query.is_empty() {
            let r = self.get_recent_display();
            tracing::trace!(
                "Emoji empty query → {} items in {:?}",
                r.len(),
                start.elapsed()
            );
            return r;
        }

        let q = query.trim().to_lowercase();

        // Fast path: if query is a known emoji glyph, surface it first.
        let mut results = self.search_engine.search_scored(&q, &self.searchable);
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let items: Vec<Item> = results
            .into_iter()
            .filter(|(_, score)| *score > 0.0)
            .map(|(item, _)| item)
            .collect();

        tracing::trace!(
            "Emoji search '{}' → {} results in {:?}",
            q,
            items.len(),
            start.elapsed()
        );
        items
    }

    fn execute(&mut self, item: &Item) -> Result<(), LatuiError> {
        if let Some(last) = self.last_action_time
            && last.elapsed() < std::time::Duration::from_millis(500)
        {
            tracing::warn!("Rate-limiting emoji copy");
            return Ok(());
        }
        self.last_action_time = Some(Instant::now());

        let glyph = item
            .metadata
            .as_ref()
            .ok_or_else(|| LatuiError::App("Missing emoji metadata".to_string()))?;

        self.copy_to_clipboard(glyph)?;
        self.record_use(glyph);

        if let Err(e) = self.save_recents() {
            tracing::error!("Failed to save emoji recents: {}", e);
        }
        Ok(())
    }

    fn record_selection(&mut self, _query: &str, _item: &Item) {}

    fn supports_preview(&self) -> bool {
        true
    }

    fn preview(&self, item: &Item) -> Option<String> {
        let glyph = item.metadata.as_ref()?;

        // Find in static db
        let row = EMOJIS.iter().find(|(g, _, _, _)| *g == glyph.as_str())?;
        let (_, name, keywords, category) = row;

        let recent_info = self
            .recents
            .iter()
            .find(|e| e.glyph.as_str() == *glyph)
            .map(|e| {
                format!(
                    "\nUsed {} time{}",
                    e.use_count,
                    if e.use_count == 1 { "" } else { "s" }
                )
            })
            .unwrap_or_default();

        Some(format!(
            "{}\n\nName: {}\nCategory: {}\nKeywords: {}{}",
            glyph,
            name,
            category,
            keywords.join(", "),
            recent_info,
        ))
    }
}

// ── Drop ───────────────────────────────────────────────────────────────────

impl Drop for EmojisMode {
    fn drop(&mut self) {
        if self.dirty
            && let Err(e) = self.save_recents()
        {
            tracing::error!("Failed to save emoji recents on drop: {}", e);
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn loaded_mode() -> EmojisMode {
        let mut m = EmojisMode::new();
        m.build_index();
        m
    }

    #[test]
    fn test_creation() {
        let m = EmojisMode::new();
        assert_eq!(m.name(), "emojis");
        assert_eq!(m.icon(), "😀");
        assert!(m.searchable.is_empty());
    }

    #[test]
    fn test_build_index_populates_all() {
        let m = loaded_mode();
        assert_eq!(m.searchable.len(), EMOJIS.len());
    }

    #[test]
    fn test_search_by_name() {
        let mut m = loaded_mode();
        let r = m.search("pizza");
        assert!(!r.is_empty());
        assert!(r[0].metadata.as_deref().unwrap().contains('🍕'));
    }

    #[test]
    fn test_search_by_keyword() {
        let mut m = loaded_mode();
        let r = m.search("laugh");
        assert!(!r.is_empty());
        // Should find face with tears of joy etc.
        assert!(r.iter().any(|i| i.metadata.as_deref() == Some("😂") || i.metadata.as_deref() == Some("🤣")));
    }

    #[test]
    fn test_search_by_category() {
        let mut m = loaded_mode();
        let r = m.search("travel");
        assert!(!r.is_empty());
    }

    #[test]
    fn test_search_empty_returns_defaults() {
        let mut m = loaded_mode();
        let r = m.search("");
        assert!(!r.is_empty());
    }

    #[test]
    fn test_search_no_match() {
        let mut m = loaded_mode();
        let r = m.search("zzzznotexisting");
        assert!(r.is_empty());
    }

    #[test]
    fn test_record_use_adds_recent() {
        let mut m = loaded_mode();
        m.record_use("😀");
        assert_eq!(m.recents.len(), 1);
        assert_eq!(m.recents[0].glyph, "😀");
        assert_eq!(m.recents[0].use_count, 1);
    }

    #[test]
    fn test_record_use_increments_duplicate() {
        let mut m = loaded_mode();
        m.record_use("😀");
        m.record_use("😀");
        assert_eq!(m.recents.len(), 1);
        assert_eq!(m.recents[0].use_count, 2);
    }

    #[test]
    fn test_record_use_promotes_to_front() {
        let mut m = loaded_mode();
        m.record_use("😀");
        m.record_use("😂");
        m.record_use("😀"); // promote
        assert_eq!(m.recents[0].glyph, "😀");
    }

    #[test]
    fn test_recents_capped() {
        let mut m = loaded_mode();
        for (glyph, _, _, _) in EMOJIS.iter().take(MAX_RECENTS + 10) {
            m.record_use(glyph);
        }
        assert!(m.recents.len() <= MAX_RECENTS);
    }

    #[test]
    fn test_dirty_flag() {
        let mut m = loaded_mode();
        assert!(!m.dirty);
        m.record_use("😀");
        assert!(m.dirty);
    }

    #[test]
    fn test_preview_known_emoji() {
        let m = loaded_mode();
        let item = Item {
            id: "emoji:🔥".into(),
            title: "🔥 fire".into(),
            search_text: "fire".into(),
            description: None,
            icon: None,
            metadata: Some("🔥".into()),
        };
        let p = m.preview(&item).unwrap();
        assert!(p.contains("fire"));
        assert!(p.contains("nature"));
        assert!(p.contains("hot"));
    }

    #[test]
    fn test_preview_unknown_emoji_returns_none() {
        let m = loaded_mode();
        let item = Item {
            id: "emoji:?".into(),
            title: "?".into(),
            search_text: "?".into(),
            description: None,
            icon: None,
            metadata: Some("🛺".into()), // not in our db
        };
        // Should be None because glyph not found in static data.
        assert!(m.preview(&item).is_none());
    }

    #[test]
    fn test_metadata_is_glyph() {
        let mut m = loaded_mode();
        let results = m.search("rocket");
        let rocket = results.iter().find(|i| i.metadata.as_deref() == Some("🚀"));
        assert!(rocket.is_some());
    }

    #[test]
    fn test_static_db_no_empty_entries() {
        for (glyph, name, keywords, category) in EMOJIS {
            assert!(!glyph.is_empty(), "Empty glyph found");
            assert!(!name.is_empty(), "Empty name for: {}", glyph);
            assert!(!keywords.is_empty(), "No keywords for: {} {}", glyph, name);
            assert!(
                !category.is_empty(),
                "Empty category for: {} {}",
                glyph,
                name
            );
        }
    }

    #[test]
    fn test_supports_preview() {
        let m = loaded_mode();
        assert!(m.supports_preview());
    }

    #[test]
    fn test_backend_name_coverage() {
        // Just ensure the enum variants compile and match correctly.
        assert_eq!(CopyBackend::Wayland, CopyBackend::Wayland);
        assert_ne!(CopyBackend::Wayland, CopyBackend::X11);
        assert_ne!(CopyBackend::X11, CopyBackend::None);
    }
}
