# Chatbot UI for flows.network

This project is derived from the [Chatbot UI](https://github.com/mckaywrigley/chatbot-ui), a project by [Mckay Wrigley](https://twitter.com/mckaywrigley). The original Chatbot UI allows you to enter API keys from cloud services (e.g., OpenAI and Azure) to chat with their LLMs. In this work, we allow it to:

* Chat with any priviately deployed LLMs, including llama2 models,
* Customize prompts, system prompts, model selection, temperature, token limits and retry limits on the server so that users have a conssistent experience across clients,
* Access vector databases, image processing, OCR, other AI models, and external web services,
* Move auth management (API keys, tokens and LLM access endpoints) to the server side for better security.

**See a [demo](https://bit.ly/learn_rust) of a Chatbot "fine tuned" with knowledge of the Rust programming.** You can ask it questions related to Rust programming. For example, if you just say "help me write a simple web server", it will give an example in Rust language complete with instructions on how to run it!

We accomplished all these by moving some of the work to the server side. The Chatbot UI is now simply a UI. It posts every message the user enters to a server, and then displays the server response. On the server side, we have [flows.network](https://flows.network/). You just need to deploy a ["flow function"](https://docs.flows.network/docs/getting-started-developer/hello-world) and configure it with your private LLM endpoints or OpenAI API keys.

> In the HTTP request to the server, the Chatbot UI uses the `x-conversation-id` header to designated a conversation ID so that the server can keep track of multiple conversations.

> The flow function is written in Rust. But do not worry, you can just reuse our template without writing a single line of code. Once you are familiar with the system, you can start to modify or write your own flow functions to customize the prompts or perform pre- post-processing work that are specific to your needs.

## How to use it

1. Deploy a flow function on [flows.network](https://flows.network/). (see examples for ChatGPT and a private llama2 LLM)
2. Copy the flow function's webhook URL in the form of `https://code.flows.network/webhook/UNIQUE-FLOW-ID`
3. Load `https://flows-chat-ui.vercel.app/?chat_url=https://code.flows.network/webhook/UNIQUE-FLOW-ID` in your browser.

That's it!

## Deploy your own front end UI

You can fork the project, customize it with your own title and text, and then deploy it on Vercel for free.

[![Deploy with Vercel](https://vercel.com/button)](https://vercel.com/new/clone?repository-url=https%3A%2F%2Fgithub.com%2Fflows-network%2Fchatbot-ui)

