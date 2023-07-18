export const Lambda = async (
  url: string,
  message: string,
  conversationName: string,
) => {
  const res = await fetch(url, {
    headers: {
      'Content-Type': 'plain/text',
      'X-Conversation-Name': conversationName
    },
    method: 'POST',
    body: message,
  });

  const result = await res.text();
  if (res.status !== 200) {
      throw {status: res.status, text: result};
  }

  return result;
};
