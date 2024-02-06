import { Conversation } from '@/types/chat';

export const updateConversation = (
  updatedConversation: Conversation,
  allConversations: Conversation[],
) => {
  const updatedConversations = allConversations.map((c) => {
    if (c.id === updatedConversation.id) {
      return updatedConversation;
    }

    return c;
  });

  saveConversation(updatedConversation);
  saveConversations(updatedConversations);

  return {
    single: updatedConversation,
    all: updatedConversations,
  };
};

export const saveConversation = (conversation: Conversation) => {
  localStorage.setItem('selectedConversation', JSON.stringify(conversation));
};

export const getSelectedConversation = () => {
  let value = localStorage.getItem('selectedConversation');
  let c: Conversation | null = value ? JSON.parse(value) : null;
  return c;
}

export const saveConversations = (conversations: Conversation[]) => {
  localStorage.setItem('conversationHistory', JSON.stringify(conversations));
};

export const getConversations = () => {
  let value = localStorage.getItem('conversationHistory');
  let c: Conversation[] = value ? JSON.parse(value) : [];
  return c;
};

export const saveShowChatbar = (showChatbar: boolean) => {
  localStorage.setItem('showChatbar', JSON.stringify(!showChatbar));
}

export const getShowChatbar = () => {
  return localStorage.getItem('showChatbar') === 'true'
}
