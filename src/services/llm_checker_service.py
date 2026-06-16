import json
import urllib.request
import urllib.error
import asyncio
from src.i18n import tr


async def send_chat_message(messages: list, settings) -> str:
    """Отправляет произвольный набор сообщений в чат."""
    if not settings.llm_base_url:
        return tr("llm_checker.send.no_url")

    data = {
        "model": settings.llm_model or "gpt-4o-mini",
        "messages": messages,
        "temperature": 0.7
    }

    url = f"{settings.llm_base_url.rstrip('/')}/chat/completions"
    headers = {"Content-Type": "application/json"}
    if settings.llm_api_key:
        headers["Authorization"] = f"Bearer {settings.llm_api_key}"

    def make_request():
        req = urllib.request.Request(url, data=json.dumps(data).encode('utf-8'), headers=headers)
        try:
            with urllib.request.urlopen(req, timeout=60) as response:
                res = json.loads(response.read().decode('utf-8'))
                return res['choices'][0]['message']['content']
        except urllib.error.URLError as e:
            body = e.read().decode('utf-8') if hasattr(e, 'read') else str(e)
            return f"API Error: {body}"
        except Exception as e:
            return f"System Error: {str(e)}"

    return await asyncio.to_thread(make_request)


async def check_patch(original_code: str, patched_code: str, settings) -> dict:
    """Возвращает словарь: {'status': str, 'reason': str, 'suggested_code': str | None}"""
    if not settings.llm_check_enabled:
        return {"status": "DISABLED", "reason": tr("llm_checker.check.disabled"), "suggested_code": None}
    if not settings.llm_base_url:
        return {"status": "ERROR", "reason": tr("llm_checker.check.no_url_short"), "suggested_code": None}

    prompt = (
        "You are an expert Python/JS/TS code reviewer.\n"
        "Review the provided patch for potential syntax errors, logic breaks, or indentation issues.\n"
        "If the patch is perfect and safe, return status 'SAFE'.\n"
        "If there are issues, or if the code could be significantly improved/fixed, return status 'ERROR', "
        "provide a 'reason', and provide the full corrected file content in 'suggested_code'.\n\n"
        "RESPOND STRICTLY IN THE FOLLOWING JSON FORMAT ONLY:\n"
        "{\n"
        "  \"status\": \"SAFE\" or \"ERROR\",\n"
        "  \"reason\": \"Your explanation...\",\n"
        "  \"suggested_code\": \"<full corrected string>\" or null\n"
        "}\n\n"
        f"--- ORIGINAL CODE ---\n{original_code}\n\n"
        f"--- PATCHED CODE (PROPOSED BY PREVIOUS STEP) ---\n{patched_code}\n"
    )

    data = {
        "model": settings.llm_model or "gpt-4o-mini",
        "messages": [{"role": "user", "content": prompt}],
        "temperature": 0.1,
        "response_format": {"type": "json_object"} # Ускоряет и форсирует JSON
    }

    url = f"{settings.llm_base_url.rstrip('/')}/chat/completions"
    headers = {"Content-Type": "application/json"}
    if settings.llm_api_key:
        headers["Authorization"] = f"Bearer {settings.llm_api_key}"

    def make_request():
        req = urllib.request.Request(url, data=json.dumps(data).encode('utf-8'), headers=headers)
        try:
            with urllib.request.urlopen(req, timeout=40) as response:
                res = json.loads(response.read().decode('utf-8'))
                content = res['choices'][0]['message']['content']
                # Пытаемся распарсить ответ как JSON
                try:
                    parsed = json.loads(content)
                    return {
                        "status": parsed.get("status", "ERROR"),
                        "reason": parsed.get("reason", "No reason provided."),
                        "suggested_code": parsed.get("suggested_code", None)
                    }
                except json.JSONDecodeError:
                    return {"status": "ERROR", "reason": tr("llm_checker.check.invalid_json"), "suggested_code": None}

        except urllib.error.URLError as e:
            body = e.read().decode('utf-8') if hasattr(e, 'read') else str(e)
            return {"status": "ERROR", "reason": f"API Error: {body}", "suggested_code": None}
        except Exception as e:
            return {"status": "ERROR", "reason": f"System Error: {str(e)}", "suggested_code": None}

    return await asyncio.to_thread(make_request)