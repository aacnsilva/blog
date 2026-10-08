/* Ask Justa: a launcher in the bottom-right corner that opens a chat panel over the page.
   Closing the panel leaves you where you were. Shared by /justa/ and /justa/privacy/. */
(() => {
  // The Cloudflare Worker that answers. The worker only accepts this site's origin.
  const ENDPOINT = "https://justa-ask.aacnsilva.workers.dev";
  const MAX_LENGTH = 800;

  const script = document.currentScript;
  const iconUrl = new URL("appicon.png", script ? script.src : location.href).href;

  const copy = {
    en: {
      launcher: "Ask Justa",
      openLabel: "Open Ask Justa",
      closeLabel: "Close Ask Justa",
      title: "Ask Justa",
      subtitle: "Plain answers about the app",
      greeting: "Hi. Ask me about plans, what is free, the Justa Plus vault, privacy, or sharing a household. English or português.",
      tryOne: "Try one",
      examples: [
        "What is free, and what does Justa Plus add?",
        "Is the Justa Plus vault a subscription?",
        "How do two iPhones share a household?",
        "What stays on my iPhone?"
      ],
      label: "Your question",
      placeholder: "Ask about Justa…",
      send: "Send",
      note: "Your household book stays on your iPhone. A question typed here is answered from Justa’s public notes. It is not saved, and there is no account to create. Each network can ask a few times an hour, and answers pause for the rest of the month once the service reaches its limit.",
      noteShort: "Questions are not saved.",
      noteMore: "How questions are handled"
    },
    pt: {
      launcher: "Perguntar",
      openLabel: "Abrir Perguntar à Justa",
      closeLabel: "Fechar Perguntar à Justa",
      title: "Perguntar à Justa",
      subtitle: "Respostas simples sobre a app",
      greeting: "Olá. Pergunte sobre planos, o que é grátis, a pasta do cofre da Justa Plus, privacidade ou partilhar uma casa. Em português ou inglês.",
      tryOne: "Experimente uma",
      examples: [
        "O que é grátis e o que acrescenta a Justa Plus?",
        "A pasta do cofre da Justa Plus é uma subscrição?",
        "Como é que dois iPhones partilham uma casa?",
        "O que fica no meu iPhone?"
      ],
      label: "A sua pergunta",
      placeholder: "Pergunte sobre a Justa…",
      send: "Enviar",
      note: "O livro da casa fica no seu iPhone. Uma pergunta escrita aqui é respondida a partir das notas públicas da Justa. Não é guardada, e não há conta para criar. Cada rede pode perguntar algumas vezes por hora, e as respostas param até ao fim do mês quando o serviço chega ao limite.",
      noteShort: "As perguntas não são guardadas.",
      noteMore: "Como as perguntas são tratadas"
    }
  };

  const messages = {
    ip_hour: {
      en: "This network has asked enough for the moment. Try again later.",
      pt: "Esta rede já perguntou o suficiente por agora. Tente mais tarde."
    },
    ip_day: {
      en: "This network has reached today’s limit.",
      pt: "Esta rede chegou ao limite de hoje."
    },
    budget: {
      en: "Answers are paused until next month.",
      pt: "As respostas ficam em pausa até ao mês que vem."
    },
    unconfigured: {
      en: "The answer service is not connected yet.",
      pt: "O serviço de respostas ainda não está ligado."
    },
    upstream: {
      en: "The answer service did not reply. Try again in a moment.",
      pt: "O serviço de respostas não respondeu. Tente outra vez daqui a pouco."
    },
    network: {
      en: "The question did not go through. Check the connection and try again.",
      pt: "A pergunta não chegou. Verifique a ligação e tente outra vez."
    },
    too_long: {
      en: "A question can be up to 800 characters.",
      pt: "A pergunta pode ter até 800 caracteres."
    }
  };

  const css = `
    .ja-launcher, .ja-panel, .ja-panel * { box-sizing: border-box; }
    .ja-launcher {
      position: fixed; right: max(20px, env(safe-area-inset-right)); bottom: max(20px, env(safe-area-inset-bottom)); z-index: 90;
      display: inline-flex; align-items: center; gap: 0.55rem;
      height: 56px; padding: 0 1.3rem 0 1rem; border: 0; border-radius: 999px;
      background: var(--text, #101B33); color: #fff;
      font: 600 0.98rem var(--sans, "Instrument Sans", -apple-system, system-ui, sans-serif);
      box-shadow: 0 16px 36px -14px rgba(16, 27, 51, 0.55), 0 2px 6px rgba(16, 27, 51, 0.12);
      cursor: pointer; transition: background 0.2s, transform 0.2s, width 0.2s, padding 0.2s;
    }
    .ja-launcher:hover { background: var(--cobalt, #1F4CC4); transform: translateY(-1px); }
    .ja-launcher:focus-visible { outline: 2px solid var(--cobalt, #1F4CC4); outline-offset: 3px; }
    .ja-launcher svg { width: 24px; height: 24px; flex: none; }
    .ja-launcher .ja-icon-close { display: none; }
    .ja-launcher[aria-expanded="true"] { width: 56px; padding: 0; justify-content: center; }
    .ja-launcher[aria-expanded="true"] .ja-icon-chat,
    .ja-launcher[aria-expanded="true"] .ja-launcher-text { display: none; }
    .ja-launcher[aria-expanded="true"] .ja-icon-close { display: block; }

    .ja-panel {
      position: fixed; z-index: 91;
      right: max(20px, env(safe-area-inset-right)); bottom: calc(max(20px, env(safe-area-inset-bottom)) + 72px);
      width: min(400px, calc(100vw - 40px)); height: min(640px, calc(100vh - 120px)); height: min(640px, calc(100dvh - 120px));
      display: flex; flex-direction: column; overflow: hidden;
      background: var(--surface, #fff); color: var(--text, #101B33);
      border: 1px solid var(--line, rgba(16, 27, 51, 0.1)); border-radius: 24px;
      box-shadow: 0 30px 70px -20px rgba(16, 27, 51, 0.45), 0 4px 14px rgba(16, 27, 51, 0.08);
      font: 400 0.97rem/1.55 var(--sans, "Instrument Sans", -apple-system, system-ui, sans-serif);
      transform-origin: bottom right;
      opacity: 0; transform: translateY(12px) scale(0.96); visibility: hidden; pointer-events: none;
      transition: opacity 0.2s ease, transform 0.25s cubic-bezier(.2,.9,.25,1.05), visibility 0s linear 0.25s;
    }
    .ja-panel[data-open="true"] {
      opacity: 1; transform: none; visibility: visible; pointer-events: auto;
      transition: opacity 0.2s ease, transform 0.25s cubic-bezier(.2,.9,.25,1.05), visibility 0s;
    }

    .ja-head {
      display: flex; align-items: center; gap: 0.75rem; flex: none;
      padding: 0.95rem 0.8rem 0.95rem 1.1rem; border-bottom: 1px solid var(--line, rgba(16, 27, 51, 0.1));
    }
    .ja-head img { width: 36px; height: 36px; border-radius: 10px; flex: none; }
    .ja-head-text { flex: 1; min-width: 0; }
    .ja-panel .ja-title { margin: 0; font: 500 1.15rem/1.2 var(--serif, "Fraunces", Georgia, serif); letter-spacing: -0.01em; }
    .ja-subtitle { margin: 0.1rem 0 0; color: var(--soft, #4E5B75); font-size: 0.82rem; }
    .ja-close {
      flex: none; width: 38px; height: 38px; border: 0; border-radius: 999px;
      display: grid; place-items: center; background: transparent; color: var(--soft, #4E5B75); cursor: pointer;
    }
    .ja-close:hover { background: var(--tint, #E3EAF8); color: var(--text, #101B33); }
    .ja-close svg { width: 20px; height: 20px; }

    .ja-body { flex: 1; min-height: 0; overflow-y: auto; -webkit-overflow-scrolling: touch; overscroll-behavior: contain; padding: 1.1rem 1.1rem 0.6rem; display: flex; flex-direction: column; gap: 0.75rem; }
    .ja-bubble { max-width: 88%; padding: 0.7rem 1rem; border-radius: 18px; white-space: pre-line; overflow-wrap: anywhere; }
    .ja-bubble p { margin: 0; color: inherit; }
    .ja-bubble.user { align-self: flex-end; background: var(--text, #101B33); color: #fff; border-bottom-right-radius: 6px; }
    .ja-bubble.assistant { align-self: flex-start; background: var(--tint, #E3EAF8); border-bottom-left-radius: 6px; }
    .ja-bubble.pending { display: inline-flex; gap: 4px; padding: 0.95rem 1.05rem; }
    .ja-bubble.pending span { width: 7px; height: 7px; border-radius: 50%; background: var(--soft, #4E5B75); opacity: 0.4; animation: ja-dot 1.2s infinite ease-in-out; }
    .ja-bubble.pending span:nth-child(2) { animation-delay: 0.15s; }
    .ja-bubble.pending span:nth-child(3) { animation-delay: 0.3s; }
    @keyframes ja-dot { 0%, 80%, 100% { opacity: 0.3; transform: none; } 40% { opacity: 1; transform: translateY(-3px); } }

    .ja-examples { display: flex; flex-direction: column; align-items: flex-start; gap: 0.45rem; margin-top: 0.2rem; }
    .ja-examples[hidden] { display: none; }
    .ja-kicker { margin: 0.2rem 0 0; font-size: 0.8rem; font-weight: 600; color: var(--soft, #4E5B75); }
    .ja-examples button {
      border: 1px solid var(--line, rgba(16, 27, 51, 0.1)); border-radius: 999px; background: var(--bg, #F4F7FC);
      padding: 0.4rem 0.9rem; font: inherit; font-size: 0.9rem; color: var(--text, #101B33); text-align: left; cursor: pointer;
    }
    .ja-examples button:hover { border-color: var(--cobalt, #1F4CC4); color: var(--cobalt, #1F4CC4); }

    .ja-foot { flex: none; padding: 0.6rem 0.9rem 0.8rem; border-top: 1px solid var(--line, rgba(16, 27, 51, 0.1)); }
    .ja-form {
      display: flex; align-items: flex-end; gap: 0.5rem;
      border: 1px solid var(--line, rgba(16, 27, 51, 0.1)); border-radius: 20px; background: var(--bg, #F4F7FC);
      padding: 0.4rem 0.4rem 0.4rem 0.95rem; transition: border-color 0.2s;
    }
    .ja-form:focus-within { border-color: var(--cobalt, #1F4CC4); }
    .ja-form textarea {
      flex: 1; min-width: 0; resize: none; border: 0; background: transparent; outline: none;
      font: inherit; font-size: 1rem; color: var(--text, #101B33); padding: 0.45rem 0; max-height: 8.5rem; line-height: 1.45;
    }
    .ja-send {
      flex: none; width: 38px; height: 38px; border: 0; border-radius: 999px; display: grid; place-items: center;
      background: var(--text, #101B33); color: #fff; cursor: pointer; transition: background 0.2s;
    }
    .ja-send:hover:not(:disabled) { background: var(--cobalt, #1F4CC4); }
    .ja-send svg { width: 18px; height: 18px; }
    .ja-panel button:disabled, .ja-panel textarea:disabled { opacity: 0.45; cursor: not-allowed; }
    /* The Justa pages style every details and summary as an FAQ row, so the note resets them. */
    .ja-panel .ja-note { margin: 0.55rem 0.2rem 0; padding: 0; border: 0; font-size: 0.76rem; color: var(--soft, #4E5B75); }
    .ja-panel .ja-note summary { display: block; cursor: pointer; list-style: none; font: inherit; }
    .ja-panel .ja-note summary::-webkit-details-marker { display: none; }
    .ja-panel .ja-note summary::after { content: none; }
    .ja-panel .ja-note summary span { text-decoration: underline; text-underline-offset: 2px; }
    .ja-panel .ja-note p { margin: 0.4rem 0 0; max-width: none; color: inherit; }
    .ja-sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); white-space: nowrap; }

    @media (max-width: 600px) {
      .ja-launcher { height: 52px; padding: 0 1.1rem 0 0.9rem; }
      .ja-launcher[aria-expanded="true"] { display: none; }
      /* The script sets top and height to the visible viewport, so the keyboard never uncovers the page. */
      .ja-panel { top: 0; right: 0; bottom: auto; left: 0; width: 100%; height: 100%; border: 0; border-radius: 0; transform: translateY(24px); transform-origin: bottom center; }
      .ja-head { padding-top: max(0.95rem, env(safe-area-inset-top)); }
      .ja-foot { padding-bottom: max(0.8rem, env(safe-area-inset-bottom)); }
      .ja-bubble { max-width: 92%; }
    }
    /* iOS ignores overflow: hidden on the page, so the page is pinned in place while the panel is open. */
    html.ja-locked, html.ja-locked body { overflow: hidden; overscroll-behavior: none; }
    html.ja-locked body { position: fixed; left: 0; right: 0; width: 100%; }
    @media (prefers-reduced-motion: reduce) {
      .ja-launcher, .ja-panel, .ja-panel[data-open="true"] { transition: none; }
      .ja-bubble.pending span { animation: none; opacity: 0.6; }
    }
  `;

  const icons = {
    chat: '<svg class="ja-icon-chat" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M4 6.5A2.5 2.5 0 016.5 4h11A2.5 2.5 0 0120 6.5v8a2.5 2.5 0 01-2.5 2.5H10l-4.2 3.2c-.5.4-1.3 0-1.3-.6V17A2.5 2.5 0 014 14.5z"/><path d="M8.5 9.5h7M8.5 12.5h4.5"/></svg>',
    close: '<svg class="ja-icon-close" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M6 9l6 6 6-6"/></svg>',
    x: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18"/></svg>',
    send: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 19V5M6 11l6-6 6 6"/></svg>'
  };

  const style = document.createElement("style");
  style.textContent = css;
  document.head.appendChild(style);

  const launcher = document.createElement("button");
  launcher.type = "button";
  launcher.className = "ja-launcher";
  launcher.setAttribute("aria-controls", "ja-panel");
  launcher.setAttribute("aria-expanded", "false");
  launcher.innerHTML = `${icons.chat}${icons.close}<span class="ja-launcher-text"></span>`;

  const panel = document.createElement("div");
  panel.className = "ja-panel";
  panel.id = "ja-panel";
  panel.setAttribute("role", "dialog");
  panel.setAttribute("aria-labelledby", "ja-title");
  panel.setAttribute("aria-hidden", "true");
  panel.innerHTML = `
    <div class="ja-head">
      <img src="${iconUrl}" alt="" width="36" height="36" />
      <div class="ja-head-text">
        <h2 class="ja-title" id="ja-title"></h2>
        <p class="ja-subtitle"></p>
      </div>
      <button type="button" class="ja-close">${icons.x}</button>
    </div>
    <div class="ja-body" role="log" aria-live="polite" aria-relevant="additions">
      <div class="ja-bubble assistant ja-greeting"><p></p></div>
      <p class="ja-kicker"></p>
      <div class="ja-examples" role="group"></div>
    </div>
    <div class="ja-foot">
      <form class="ja-form">
        <label class="ja-sr" for="ja-question"></label>
        <textarea id="ja-question" name="question" rows="1" maxlength="${MAX_LENGTH}" required></textarea>
        <button type="submit" class="ja-send">${icons.send}<span class="ja-sr ja-send-text"></span></button>
      </form>
      <details class="ja-note">
        <summary></summary>
        <p></p>
      </details>
    </div>
  `;

  document.body.append(panel, launcher);

  const $ = (selector) => panel.querySelector(selector);
  const body = $(".ja-body");
  const kicker = $(".ja-kicker");
  const examples = $(".ja-examples");
  const form = $(".ja-form");
  const question = $("#ja-question");
  const send = $(".ja-send");
  const closeButton = $(".ja-close");

  const conversation = [];
  let sending = false;
  let closed = false;
  let statusChecked = false;
  let returnFocus = null;

  const currentLang = () => (document.body.dataset.lang === "pt" ? "pt" : "en");
  const say = (code) => (messages[code] || messages.network)[currentLang()];
  const isOpen = () => panel.dataset.open === "true";
  const isNarrow = () => window.matchMedia("(max-width: 600px)").matches;

  const render = () => {
    const text = copy[currentLang()];
    launcher.querySelector(".ja-launcher-text").textContent = text.launcher;
    launcher.setAttribute("aria-label", isOpen() ? text.closeLabel : text.openLabel);
    closeButton.setAttribute("aria-label", text.closeLabel);
    $(".ja-title").textContent = text.title;
    $(".ja-subtitle").textContent = text.subtitle;
    $(".ja-greeting p").textContent = text.greeting;
    kicker.textContent = text.tryOne;
    examples.setAttribute("aria-label", text.tryOne);
    examples.replaceChildren(...text.examples.map((example) => {
      const button = document.createElement("button");
      button.type = "button";
      button.textContent = example;
      button.disabled = sending || closed;
      return button;
    }));
    $("label").textContent = text.label;
    question.placeholder = text.placeholder;
    $(".ja-send-text").textContent = text.send;
    $(".ja-note summary").innerHTML = `${text.noteShort} <span></span>`;
    $(".ja-note summary span").textContent = text.noteMore;
    $(".ja-note p").textContent = text.note;
  };

  const scrollToEnd = () => { body.scrollTop = body.scrollHeight; };

  const addBubble = (role, text) => {
    const bubble = document.createElement("div");
    bubble.className = `ja-bubble ${role}`;
    const paragraph = document.createElement("p");
    paragraph.textContent = text;
    bubble.appendChild(paragraph);
    body.appendChild(bubble);
    scrollToEnd();
    return bubble;
  };

  const addPending = () => {
    const bubble = document.createElement("div");
    bubble.className = "ja-bubble assistant pending";
    bubble.setAttribute("aria-busy", "true");
    bubble.innerHTML = "<span></span><span></span><span></span>";
    body.appendChild(bubble);
    scrollToEnd();
    return bubble;
  };

  const lock = (busy) => {
    sending = busy;
    const locked = busy || closed;
    send.disabled = locked;
    question.disabled = locked;
    examples.querySelectorAll("button").forEach((button) => { button.disabled = locked; });
  };

  const grow = () => {
    question.style.height = "auto";
    question.style.height = `${question.scrollHeight}px`;
  };

  const checkStatus = async () => {
    if (statusChecked) return;
    statusChecked = true;
    try {
      const response = await fetch(`${ENDPOINT}/status`);
      if (!response.ok) return;
      const payload = await response.json();
      if (payload.accepting === false) {
        closed = true;
        addBubble("assistant", say("budget"));
        lock(false);
      }
    } catch (error) {
      // A failed status check is not a reason to hide the box. Asking still works or explains itself.
    }
  };

  const ask = async (text) => {
    const asked = text.trim();
    if (!asked || sending || closed) return;
    if (asked.length > MAX_LENGTH) {
      addBubble("assistant", say("too_long"));
      return;
    }

    kicker.hidden = true;
    examples.hidden = true;
    addBubble("user", asked);
    question.value = "";
    grow();
    conversation.push({ role: "user", content: asked });
    const pending = addPending();
    lock(true);

    try {
      const response = await fetch(`${ENDPOINT}/ask`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ messages: conversation.slice(-6) })
      });
      const payload = await response.json().catch(() => ({}));
      pending.remove();
      if (!response.ok) {
        conversation.pop();
        if (payload.error === "budget") closed = true;
        addBubble("assistant", messages[payload.error] ? say(payload.error) : say("upstream"));
        return;
      }
      const answer = typeof payload.answer === "string" && payload.answer.trim()
        ? payload.answer.trim()
        : say("upstream");
      addBubble("assistant", answer);
      conversation.push({ role: "assistant", content: answer.slice(0, 1500) });
    } catch (error) {
      pending.remove();
      conversation.pop();
      addBubble("assistant", say("network"));
    } finally {
      lock(false);
      if (!closed && isOpen()) question.focus();
    }
  };

  const open = () => {
    if (isOpen()) return;
    returnFocus = document.activeElement instanceof HTMLElement && document.activeElement !== document.body
      ? document.activeElement
      : launcher;
    panel.dataset.open = "true";
    panel.setAttribute("aria-hidden", "false");
    launcher.setAttribute("aria-expanded", "true");
    fitToViewport();
    render();
    checkStatus();
    scrollToEnd();
    // Wait for the panel to become visible before moving focus into it.
    requestAnimationFrame(() => { if (!question.disabled) question.focus({ preventScroll: true }); });
  };

  const close = () => {
    if (!isOpen()) return;
    panel.dataset.open = "false";
    panel.setAttribute("aria-hidden", "true");
    launcher.setAttribute("aria-expanded", "false");
    fitToViewport();
    render();
    const target = returnFocus && document.contains(returnFocus) ? returnFocus : launcher;
    target.focus({ preventScroll: true });
  };

  // On a phone the panel covers the screen. The page behind is pinned, and the panel follows
  // the visible viewport, which shrinks and moves when the keyboard opens.
  const viewport = window.visualViewport;
  let pinnedScroll = 0;

  const pinPage = () => {
    if (document.documentElement.classList.contains("ja-locked")) return;
    pinnedScroll = window.scrollY;
    document.body.style.top = `-${pinnedScroll}px`;
    document.documentElement.classList.add("ja-locked");
  };

  const unpinPage = () => {
    if (!document.documentElement.classList.contains("ja-locked")) return;
    document.documentElement.classList.remove("ja-locked");
    document.body.style.top = "";
    window.scrollTo({ top: pinnedScroll, behavior: "instant" });
  };

  const fitToViewport = () => {
    if (!isOpen() || !isNarrow()) {
      unpinPage();
      panel.style.top = "";
      panel.style.height = "";
      return;
    }
    pinPage();
    if (!viewport) return;
    const atEnd = body.scrollHeight - body.scrollTop - body.clientHeight < 24;
    panel.style.top = `${viewport.offsetTop}px`;
    panel.style.height = `${viewport.height}px`;
    if (atEnd) scrollToEnd();
  };

  const handleLauncher = () => (isOpen() ? close() : open());
  const handleSubmit = (event) => {
    event.preventDefault();
    ask(question.value);
  };
  const handleKeyDown = (event) => {
    if (event.key !== "Enter" || event.shiftKey || event.isComposing) return;
    event.preventDefault();
    form.requestSubmit();
  };
  const handleExample = (event) => {
    const button = event.target.closest("button");
    if (button) ask(button.textContent);
  };
  const handleEscape = (event) => {
    if (event.key === "Escape" && isOpen()) close();
  };
  // Links to the old Ask page open the panel instead, so the page stays put.
  const handleAskLink = (event) => {
    const link = event.target.closest("a[href]");
    if (!link || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || event.button !== 0) return;
    const url = new URL(link.href, location.href);
    if (url.origin !== location.origin || !/\/justa\/ask\/?$/.test(url.pathname)) return;
    event.preventDefault();
    open();
  };

  launcher.addEventListener("click", handleLauncher);
  closeButton.addEventListener("click", close);
  form.addEventListener("submit", handleSubmit);
  question.addEventListener("keydown", handleKeyDown);
  question.addEventListener("input", grow);
  examples.addEventListener("click", handleExample);
  document.addEventListener("keydown", handleEscape);
  document.addEventListener("click", handleAskLink);
  window.addEventListener("resize", fitToViewport);
  if (viewport) {
    viewport.addEventListener("resize", fitToViewport);
    viewport.addEventListener("scroll", fitToViewport);
  }
  new MutationObserver(render).observe(document.body, { attributes: true, attributeFilter: ["data-lang"] });

  render();

  // /justa/ask/ sends people here with ?ask so the panel opens over the landing page.
  const params = new URLSearchParams(location.search);
  if (params.has("ask")) {
    params.delete("ask");
    const query = params.toString();
    history.replaceState(null, "", `${location.pathname}${query ? `?${query}` : ""}${location.hash}`);
    open();
  }
})();
