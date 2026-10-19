import json
import re
import ollama
from typing import List, Dict, Set, Any


class TextProcessor:
    def __init__(self):
        self.punctuation_pattern = re.compile(r'[.,!?:;"\'\-«»—()]')
        self.word_pattern = re.compile(r'\w+', re.UNICODE)

    def clean_text(self, text: str) -> str:
        return self.punctuation_pattern.sub(' ', text)

    def tokenize(self, text: str) -> List[str]:
        cleaned = self.clean_text(text)
        words = self.word_pattern.findall(cleaned)
        return [word.lower() for word in words if word.strip()]


class TranslationService:
    def __init__(self, model: str = "gemma4:e4b", temperature: float = 0.1, max_retries: int = 3):
        self.model = model
        self.temperature = temperature
        self.max_retries = max_retries
        self.failed_words: List[str] = []

    def parse_translation(self, response: str) -> List[str]:
        cleaned = response.strip()
        
        if cleaned.startswith("```") and cleaned.endswith("```"):
            cleaned = cleaned[3:-3].strip()
        elif cleaned.startswith('`') and cleaned.endswith('`'):
            cleaned = cleaned[1:-1]
        if cleaned.startswith('"') and cleaned.endswith('"'):
            cleaned = cleaned[1:-1]
        
        cleaned = cleaned.strip().rstrip('.').rstrip(',')
        
        parts = cleaned.split(',')
        result = []
        for part in parts:
            word = part.strip().lower().rstrip('.').rstrip(',').strip()
            if word and len(word) > 0:
                result.append(word)
        
        if 1 <= len(result) <= 3:
            return result
        return []

    def translate_word(self, word: str) -> List[str]:
        prompt = f"In 1-3 words comma separated give me the closest english words to '{word}'. You MUST stick to the format asked. Output ONLY the words."
        
        for attempt in range(self.max_retries):
            try:
                response = ollama.chat(
                    model=self.model,
                    messages=[{"role": "user", "content": prompt}],
                    options={"temperature": self.temperature},
                    think=False
                )
                content = response.get('message', {}).get('content', '').strip()
                translations = self.parse_translation(content)
                
                if translations:
                    return translations
                    
            except Exception as e:
                pass

        print("Failed word: ", word)
        self.failed_words.append(word)
        return ["[translation missing]"]

    def translate_words_batch(self, words: Set[str]) -> Dict[str, List[str]]:
        translations = {}
        for idx,word in enumerate(words):
            translations[word] = self.translate_word(word)
            if idx % 100 == 0:
                print(f"translation percentage done {idx/len(words) * 100:.4f}%")
        return translations


class PipelineRunner:
    def __init__(self, input_file: str = "rst.json", translations_file: str = "translations.json", 
                 states_file: str = "word_states.json", failed_log: str = "failed_words.log"):
        self.input_file = input_file
        self.translations_file = translations_file
        self.states_file = states_file
        self.failed_log = failed_log
        self.text_processor = TextProcessor()
        self.translation_service = TranslationService(model="gemma4:e4b")

    def run(self):
        all_words: Set[str] = set()
        unique_words: Set[str] = set()
        
        with open(self.input_file, 'r', encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    verse = json.loads(line)
                    text = verse.get('text', '')
                    tokens = self.text_processor.tokenize(text)
                    for token in tokens:
                        if token and not token.isdigit():
                            all_words.add(token)
                            unique_words.add(token)
                except json.JSONDecodeError:
                    continue
        
        print(f"Found {len(unique_words)} unique words to translate")
        
        translations = self.translation_service.translate_words_batch(unique_words)
        
        output_translations = {
            "translations": [
                {"word": word, "closestWords": trans}
                for word, trans in sorted(translations.items())
            ]
        }
        
        with open(self.translations_file, 'w', encoding='utf-8') as f:
            json.dump(output_translations, f, indent=2, ensure_ascii=False)
        
        print(f"Saved translations to {self.translations_file}")
        
        states = {
            "states": [
                [word, "unseen"]
                for word in sorted(all_words)
            ]
        }
        
        with open(self.states_file, 'w', encoding='utf-8') as f:
            json.dump(states, f, indent=2, ensure_ascii=False)
        
        print(f"Saved word states to {self.states_file}")
        
        if self.translation_service.failed_words:
            with open(self.failed_log, 'w', encoding='utf-8') as f:
                for word in self.translation_service.failed_words:
                    f.write(f"{word}\n")
            print(f"Failed words logged to {self.failed_log}")
        
        print(f"Translation complete. Translated {len(unique_words)} unique words.")
        if self.translation_service.failed_words:
            print(f"Failed to translate {len(self.translation_service.failed_words)} words.")


if __name__ == "__main__":
    pipeline = PipelineRunner()
    pipeline.run()
