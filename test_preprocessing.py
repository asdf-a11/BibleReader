import pytest
import json
import os
from unittest.mock import patch, MagicMock
from preprocessing import TextProcessor, TranslationService, PipelineRunner


class TestTextProcessor:
    def setup_method(self):
        self.processor = TextProcessor()

    def test_tokenize_sentence(self):
        text = "В начале, сотворил Бог!"
        tokens = self.processor.tokenize(text)
        assert tokens == ["в", "начале", "сотворил", "бог"]

    def test_tokenize_with_various_punctuation(self):
        text = "Привет, мир! Как дела? Хорошо: хорошо."
        tokens = self.processor.tokenize(text)
        assert tokens == ["привет", "мир", "как", "дела", "хорошо", "хорошо"]

    def test_tokenize_cyrillic_and_latin(self):
        text = "Test привет World мир"
        tokens = self.processor.tokenize(text)
        assert tokens == ["test", "привет", "world", "мир"]

    def test_tokenize_empty_string(self):
        tokens = self.processor.tokenize("")
        assert tokens == []

    def test_tokenize_numbers(self):
        tokens = self.processor.tokenize("Текст 123 цифры")
        assert "123" in tokens

    def test_lowercase_conversion(self):
        text = "ПРИВЕТ МИР"
        tokens = self.processor.tokenize(text)
        assert tokens == ["привет", "мир"]


class TestTranslationService:
    def setup_method(self):
        self.service = TranslationService(model="gemma4:12b", temperature=0.1, max_retries=3)

    def test_translation_parser_valid(self):
        response = "hello, hi, greetings"
        result = self.service.parse_translation(response)
        assert result == ["hello", "hi", "greetings"]

    def test_translation_parser_formatting_cleanup(self):
        response = " hello , hi . "
        result = self.service.parse_translation(response)
        assert result == ["hello", "hi"]

    def test_translation_parser_trailing_period(self):
        response = "hello, hi"
        result = self.service.parse_translation(response)
        assert result == ["hello", "hi"]

    def test_translation_parser_single_word(self):
        response = "hello"
        result = self.service.parse_translation(response)
        assert result == ["hello"]

    def test_translation_parser_markdown_code_block(self):
        response = "```hello, hi```"
        result = self.service.parse_translation(response)
        assert result == ["hello", "hi"]

    def test_translation_parser_excessive_words(self):
        response = "hello, hi, greetings, world"
        result = self.service.parse_translation(response)
        assert result == []

    def test_translation_parser_empty_response(self):
        response = ""
        result = self.service.parse_translation(response)
        assert result == []

    def test_translation_parser_whitespace_only(self):
        response = "   "
        result = self.service.parse_translation(response)
        assert result == []

    @patch('ollama.chat')
    def test_ollama_retry_and_fallback(self, mock_ollama):
        mock_response = MagicMock()
        mock_response.configure_mock(
            message=MagicMock(content="invalid response with too many words here")
        )
        mock_ollama.return_value = mock_response
        
        service = TranslationService(model="gemma4:12b", temperature=0.1, max_retries=3)
        result = service.translate_word("тест")
        
        assert result == ["[translation missing]"]
        assert "тест" in service.failed_words

    @patch('ollama.chat')
    def test_successful_translation(self, mock_ollama):
        mock_response = {
            'message': {'content': 'hello, hi'}
        }
        mock_ollama.return_value = mock_response
        
        service = TranslationService(model="gemma4:12b", temperature=0.1, max_retries=3)
        result = service.translate_word("привет")
        
        assert result == ["hello", "hi"]


class TestPipelineRunner:
    def setup_method(self):
        self.runner = PipelineRunner(
            input_file="rst.json",
            translations_file="test_translations.json",
            states_file="test_word_states.json",
            failed_log="test_failed_words.log"
        )

    def test_word_states_schema(self):
        all_words = {"привет", "мир", "тест"}
        states = {
            "states": [
                [word, "unseen"]
                for word in sorted(all_words)
            ]
        }
        
        assert "states" in states
        for word, state in states["states"]:
            assert state == "unseen"

    def test_valid_json_output(self):
        translations = {
            "translations": [
                {"word": "привет", "closestWords": ["hello", "hi"]}
            ]
        }
        
        with open("test_output.json", "w", encoding="utf-8") as f:
            json.dump(translations, f, ensure_ascii=False)
        
        with open("test_output.json", "r", encoding="utf-8") as f:
            loaded = json.load(f)
        
        assert "translations" in loaded
        assert loaded["translations"][0]["word"] == "привет"
        assert loaded["translations"][0]["closestWords"] == ["hello", "hi"]
        
        os.remove("test_output.json")

    def test_extract_unique_words(self):
        test_data = [
            '{"chapter":1,"verse":1,"text":"В начале сотворил Бог","translation_id":"RST","book_id":"Gen","book_name":"Genesis"}',
            '{"chapter":1,"verse":2,"text":"В начале был свет","translation_id":"RST","book_id":"Gen","book_name":"Genesis"}'
        ]
        
        unique_words = set()
        for line in test_data:
            verse = json.loads(line)
            text = verse.get('text', '')
            tokens = self.runner.text_processor.tokenize(text)
            for token in tokens:
                unique_words.add(token)
        
        assert unique_words == {"в", "начале", "сотворил", "бог", "был", "свет"}


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
