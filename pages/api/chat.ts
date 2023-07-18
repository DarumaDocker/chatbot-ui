import { Lambda } from '@/utils/server/lambda';

import { ChatBody } from '@/types/chat';

export const config = {
  runtime: 'edge',
};

const handler = async (req: Request): Promise<Response> => {
  try {
    const { messages, url, conversationName } = (await req.json()) as ChatBody;

    let messageToSend = messages[messages.length - 1].content;

    const resp = await Lambda(url, messageToSend, conversationName);

    return new Response(resp);
  } catch (error) {
    if (typeof error == 'string') {
      return new Response(error, { status: 500, statusText: error });
    } else if (typeof error == 'object' && error && (error as any).status) {
      let err = error as any;
      if (err.text.length > 256) {
        return new Response(err.text, { status: err.status });
      } else {
        return new Response(err.text, { status: err.status, statusText: err.text });
      }
    } else {
      return new Response('Error', { status: 500 });
    }
  }
};

export default handler;
