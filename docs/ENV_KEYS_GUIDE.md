# API Keys & Environment Guide

AutoShorts uses several third-party APIs for transcription and AI analysis. You can configure these directly in the application's settings, and they are stored securely in your local storage.

Here is a guide on how to obtain all the optional API keys you might need.

---

## 1. Deepgram (Transcription)
Deepgram is used for highly accurate and fast audio transcription.
1. Go to the [Deepgram Console](https://console.deepgram.com/).
2. Create an account or log in.
3. In the left sidebar, click on **API Keys**.
4. Click **Create a New API Key**.
5. Give it a name (e.g., "AutoShorts") and assign it "Member" privileges.
6. Copy the generated key. *You won't be able to see it again!*

---

## 2. Anthropic / Claude (LLM)
Claude is excellent at analyzing transcripts to find engaging hooks.
1. Go to the [Anthropic Console](https://console.anthropic.com/).
2. Log in and navigate to the **API Keys** section in your profile/settings.
3. Click **Create Key**.
4. Name it "AutoShorts" and copy the key (it starts with `sk-ant-`).
*(Note: You must have a funded Anthropic API account for this to work).*

---

## 3. OpenAI / ChatGPT (LLM)
1. Go to the [OpenAI Developer Platform](https://platform.openai.com/api-keys).
2. Log in and click **Create new secret key**.
3. Name it "AutoShorts" and copy the key (it starts with `sk-proj-`).

---

## 4. DeepSeek (LLM)
DeepSeek is a powerful and very cost-effective LLM alternative.
1. Go to the [DeepSeek Platform](https://platform.deepseek.com/).
2. Create an account.
3. Navigate to the **API Keys** tab on the left.
4. Click **Create API Key**, copy it, and save it.

---

## 5. Google Gemini (LLM)
1. Go to [Google AI Studio](https://aistudio.google.com/app/apikey).
2. Sign in with your Google account.
3. Click **Create API key**.
4. Select a project (or create a new one) and click create. Copy your key.

---

## 6. OpenRouter (Multi-LLM access)
OpenRouter lets you access dozens of models (like Meta Llama, Mistral, etc.) with a single API key.
1. Go to [OpenRouter](https://openrouter.ai/).
2. Create an account and go to **Keys**.
3. Click **Create Key**.
4. Copy the key generated for you.

---

## Local Models (No API Key Required)
If you do not want to use external APIs, you can run AutoShorts entirely offline!
- **Local Transcription**: Uses built-in processing.
- **Local LLM**: AutoShorts supports local models like `qwen2.5` running on your own hardware via tools like [Ollama](https://ollama.com/). No API key is needed.
