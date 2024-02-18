import { ChatBody } from "@/types/chat";
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'


export async function sendChatBody(chatbody: ChatBody): Promise<ReadableStream<string>> {
    console.log(chatbody);
    if (window.__TAURI__ != undefined) {
        const readableStream = new ReadableStream({
            start(controller) {
                let unlisten = listen(`output/${chatbody.conversationName}`, (event) => {
                    console.log('event', event)
                    if (event.payload === null) {
                        controller.close()
                        unlisten.then(unlisten => unlisten());
                    } else {
                        controller.enqueue(event.payload)
                    }
                })
            }
        });

        invoke("send_chat_body", {
            chatBody: chatbody
        });


        return readableStream;
    } else {
        const dataChunks = [chatbody.messages[chatbody.messages.length - 1].content]
        const readableStream = new ReadableStream({
            start(controller) {
                dataChunks.forEach((chunk: string) => {
                    controller.enqueue(chunk);
                });
                setTimeout(() => {
                    controller.close();
                }, 2000);
            }
        });

        return readableStream;
    }
}