"""Translate the Russian Synodal bible into English, sentence by sentence.

Every verse in rst.json is split into sentences and each sentence is sent to a
local Ollama model. The result is written to verse_translations.json, which the
Rust reader loads to show the English under each Russian sentence.

The run is resumable: progress is saved every SAVE_EVERY_SECONDS, and starting
the script again skips every sentence already in the output file. Ctrl+C saves
before exiting.

    python3 generate_verse_translations.py              # translate everything
    python3 generate_verse_translations.py --limit 50   # try a few sentences
    python3 generate_verse_translations.py --retry-failed
"""

import argparse
import json
import os
import random
import re
import sys
import time
from typing import Any, Dict, List, Optional, Tuple

import ollama

MODEL = "gemma4:e4b"
INPUT_FILE = "rst.json"
OUTPUT_FILE = "verse_translations.json"
SAVE_EVERY_SECONDS = 60
REPORT_EVERY_SECONDS = 150

SYSTEM_PROMPT = (
    "You translate sentences from the Russian Synodal Bible into clear, literal English "
    "for a student learning Russian. Translate ONLY the sentence you are given, not the "
    "rest of the verse. Keep proper names. Do not add notes, explanations, alternatives "
    "or Russian text. Reply with the English translation and nothing else."
)

# A sentence ends at . ! ? or … (possibly followed by closing quotes/brackets)
# and whitespace. Whatever is left at the end of a verse is a sentence too.
SENTENCE_END = re.compile(r'[.!?…]+[»”"’)\]]*\s+')
CYRILLIC = re.compile(r"[Ѐ-ӿ]")
# Phrases a model uses when it talks about the translation instead of giving it.
CHATTER = re.compile(
    r"^(here is|here's)\b.*\btranslation|^(sure|certainly|okay)\b|\bnote:|"
    r"\b(alternatively|translates to|translated as|the sentence)\b|\(lit\.",
    re.IGNORECASE,
)

Key = Tuple[str, int, int, int]


def split_sentences(text: str) -> List[str]:
    text = " ".join(text.split())
    sentences = []
    start = 0
    for match in SENTENCE_END.finditer(text):
        sentences.append(text[start:match.end()].strip())
        start = match.end()
    if start < len(text):
        sentences.append(text[start:].strip())
    return [s for s in sentences if s]


def clean_response(response: str) -> str:
    cleaned = response.strip()
    if cleaned.startswith("```") and cleaned.endswith("```"):
        cleaned = cleaned[3:-3].strip()
    # Drop a label such as "English:" or "Translation:".
    cleaned = re.sub(r"^(english|translation)\s*:\s*", "", cleaned, flags=re.IGNORECASE)
    if len(cleaned) >= 2 and cleaned[0] == cleaned[-1] and cleaned[0] in "\"'`":
        cleaned = cleaned[1:-1]
    return cleaned.strip()


def problem_with(russian: str, english: str) -> Optional[str]:
    """Why `english` is not an acceptable translation of `russian`, or None."""
    if not english:
        return "empty"
    if CYRILLIC.search(english):
        return "contains Russian"
    if "\n" in english:
        return "more than one line"
    if CHATTER.search(english):
        return "commentary"
    ratio = len(english) / max(len(russian), 1)
    if len(english) > 30 and not 0.3 <= ratio <= 3.0:
        return f"suspicious length ({ratio:.1f}x)"
    return None


class Translator:
    def __init__(self, model: str = MODEL, temperature: float = 0.1, max_retries: int = 4):
        self.model = model
        self.temperature = temperature
        self.max_retries = max_retries

    def translate(self, sentence: str, verse: str) -> Tuple[Optional[str], str]:
        """Returns (translation or None, last problem seen)."""
        if sentence == verse:
            user = f"Sentence to translate: {sentence}"
        else:
            user = f"Verse (context only): {verse}\nSentence to translate: {sentence}"
        problem = "no attempt"
        for attempt in range(self.max_retries):
            # Retry with a little randomness so the model does not repeat itself.
            temperature = self.temperature if attempt == 0 else 0.4
            try:
                response = ollama.chat(
                    model=self.model,
                    messages=[
                        {"role": "system", "content": SYSTEM_PROMPT},
                        {"role": "user", "content": user},
                    ],
                    options={"temperature": temperature},
                    think=False,
                )
            except Exception as e:  # connection errors, model not loaded, ...
                problem = f"ollama error: {e}"
                time.sleep(2)
                continue
            english = clean_response(response["message"]["content"])
            problem = problem_with(sentence, english)
            if problem is None:
                return english, ""
        return None, problem


def load_jobs(input_file: str) -> List[Dict[str, Any]]:
    """Every sentence of the bible, in reading order."""
    jobs = []
    with open(input_file, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            verse = json.loads(line)
            text = " ".join(verse["text"].split())
            for index, sentence in enumerate(split_sentences(text)):
                jobs.append({
                    "book_id": verse["book_id"],
                    "chapter": verse["chapter"],
                    "verse": verse["verse"],
                    "index": index,
                    "russian": sentence,
                    "verse_text": text,
                })
    return jobs


def key_of(entry: Dict[str, Any]) -> Key:
    return (entry["book_id"], entry["chapter"], entry["verse"], entry["index"])


class Store:
    """The output file: done sentences plus the ones the model kept failing on."""

    def __init__(self, path: str, model: str):
        self.path = path
        self.model = model
        self.done: Dict[Key, Dict[str, Any]] = {}
        self.failed: Dict[Key, Dict[str, Any]] = {}
        if os.path.exists(path):
            with open(path, "r", encoding="utf-8") as f:
                data = json.load(f)
            # Older files used a different layout; only resume from this one.
            for entry in data.get("sentences", []):
                self.done[key_of(entry)] = entry
            for entry in data.get("failed", []):
                self.failed[key_of(entry)] = entry

    def save(self, order: Dict[Key, int]) -> None:
        position = lambda e: order.get(key_of(e), len(order))
        data = {
            "model": self.model,
            "sentences": sorted(self.done.values(), key=position),
            "failed": sorted(self.failed.values(), key=position),
        }
        # Write to a temporary file first so a crash never leaves half a file.
        tmp = self.path + ".tmp"
        with open(tmp, "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, indent=1)
        os.replace(tmp, self.path)


def format_duration(seconds: float) -> str:
    seconds = int(seconds)
    hours, rest = divmod(seconds, 3600)
    minutes, seconds = divmod(rest, 60)
    if hours:
        return f"{hours}h {minutes:02d}m"
    return f"{minutes}m {seconds:02d}s"


def run(args: argparse.Namespace) -> None:
    jobs = load_jobs(args.input)
    order = {key_of(job): i for i, job in enumerate(jobs)}
    store = Store(args.output, args.model)
    if store.done and os.path.exists(args.output):
        print(f"Resuming: {len(store.done)} sentences already translated.")

    todo = [job for job in jobs if key_of(job) not in store.done]
    if args.retry_failed:
        # The failed sentences first, so they are retried even with --limit.
        todo.sort(key=lambda job: key_of(job) not in store.failed)
    else:
        todo = [job for job in todo if key_of(job) not in store.failed]
    if args.sample:
        todo = random.Random(args.seed).sample(todo, min(args.sample, len(todo)))
    if args.limit:
        todo = todo[: args.limit]

    total_sentences = len(jobs)
    print(f"{len(todo)} sentences to translate ({total_sentences} in the bible).")
    if not todo:
        return

    translator = Translator(model=args.model)
    started = time.time()
    last_save = last_report = started
    finished = 0

    def report() -> None:
        elapsed = time.time() - started
        rate = finished / elapsed if elapsed > 0 else 0
        remaining = (len(todo) - finished) / rate if rate > 0 else 0
        done = len(store.done)
        print(
            f"[{time.strftime('%H:%M:%S')}] {done}/{total_sentences} sentences done "
            f"({100 * done / total_sentences:.1f}%), this run {finished}/{len(todo)} "
            f"at {rate * 60:.0f}/min, {len(store.failed)} failed, "
            f"about {format_duration(remaining)} left",
            flush=True,
        )

    try:
        for job in todo:
            key = key_of(job)
            english, problem = translator.translate(job["russian"], job["verse_text"])
            entry = {k: job[k] for k in ("book_id", "chapter", "verse", "index", "russian")}
            if english is None:
                entry["problem"] = problem
                store.failed[key] = entry
                print(f"  failed {job['book_id']} {job['chapter']}:{job['verse']} "
                      f"({problem}): {job['russian']}", flush=True)
            else:
                entry["english"] = english
                store.done[key] = entry
                store.failed.pop(key, None)
                if args.verbose:
                    print(f"  {job['russian']}\n    -> {english}", flush=True)
            finished += 1

            now = time.time()
            if now - last_save >= SAVE_EVERY_SECONDS:
                store.save(order)
                last_save = now
            if now - last_report >= REPORT_EVERY_SECONDS:
                report()
                last_report = now
    except KeyboardInterrupt:
        print("\nInterrupted, saving progress...")
    finally:
        store.save(order)
        report()
        print(f"Saved to {args.output}. Run the script again to continue.")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--input", default=INPUT_FILE)
    parser.add_argument("--output", default=OUTPUT_FILE)
    parser.add_argument("--model", default=MODEL)
    parser.add_argument("--limit", type=int, help="translate at most this many sentences")
    parser.add_argument("--sample", type=int, help="translate this many random sentences (for testing)")
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--retry-failed", action="store_true",
                        help="also retry sentences the model failed on before")
    parser.add_argument("--verbose", action="store_true", help="print every translation")
    run(parser.parse_args())


if __name__ == "__main__":
    sys.exit(main())
