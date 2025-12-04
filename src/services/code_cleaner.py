"""Code cleaning and minification service."""
import re
from typing import Dict, Any


class CodeCleaner:
    """Сервис очистки и минификации кода."""
    
    @staticmethod
    def remove_comments(text: str, extension: str) -> str:
        """
        Базовое удаление комментариев на основе регулярных выражений.
        
        Args:
            text: Source code text
            extension: File extension to determine comment style
            
        Returns:
            Text with comments removed
        """
        # C-style comments (JS, TS, C++, Java, CSS, SCSS)
        if extension in ['.js', '.ts', '.vue', '.jsx', '.tsx', '.css', '.scss', '.java', '.cpp', '.c', '.h', '.go']:
            # Удаляем блочные /* */ и строчные //
            pattern = r"(\".*?\"|\'.*?\')|(/\*.*?\*/|//[^\r\n]*$)"
            # Функция замены проверяет, является ли совпадение строкой кода или комментарием
            def replacer(match):
                if match.group(1): 
                    return match.group(1)  # Это строка в кавычках, оставляем
                return ""  # Это комментарий, удаляем
            
            return re.sub(pattern, replacer, text, flags=re.MULTILINE|re.DOTALL)
        
        # Python-style comments
        elif extension in ['.py', '.sh', '.yaml', '.yml', '.rb']:
             # Удаляем #, но не внутри строк
             pattern = r"(\".*?\"|\'.*?\')|(#.*$)"
             def replacer(match):
                if match.group(1): 
                    return match.group(1)
                return ""
             return re.sub(pattern, replacer, text, flags=re.MULTILINE)
        
        return text

    @staticmethod
    def minify_whitespace(text: str) -> str:
        """
        Удаляет лишние пустые строки и пробелы в начале/конце строк.
        
        Args:
            text: Source code text
            
        Returns:
            Minified text
        """
        lines = [line.strip() for line in text.splitlines()]
        # Фильтруем пустые строки, оставляя структуру минимально читаемой
        non_empty_lines = [line for line in lines if line]
        return "\n".join(non_empty_lines)

    def process(self, text: str, ext: str, options: Dict[str, Any]) -> str:
        """
        Process text with cleaning and minification options.
        
        Args:
            text: Source code text
            ext: File extension
            options: Dictionary with processing options
            
        Returns:
            Processed text
        """
        if options.get('remove_comments'):
            text = self.remove_comments(text, ext)
        
        if options.get('minify'):
            text = self.minify_whitespace(text)
            
        return text