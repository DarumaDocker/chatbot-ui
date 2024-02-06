import { ChatBody } from "@/types/chat";

export async function sendChatBody(chatbody: ChatBody): Promise<ReadableStream<string>> {
    console.log(chatbody.messages);
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