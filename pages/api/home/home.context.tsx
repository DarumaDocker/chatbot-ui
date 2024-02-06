import { Dispatch, createContext } from 'react';

import { ActionType } from '@/hooks/useCreateReducer';

import { Conversation } from '@/types/chat';
import { KeyValuePair } from '@/types/data';
import { FolderType } from '@/types/folder';

import { HomeInitialState, initialState } from './home.state';

export interface HomeContextProps {
  state: HomeInitialState;
  dispatch: Dispatch<ActionType<HomeInitialState>>;
  handleNewConversation: () => void;
  handleCreateFolder: (name: string, type: FolderType) => void;
  handleDeleteFolder: (folderId: string) => void;
  handleUpdateFolder: (folderId: string, name: string) => void;
  handleSelectConversation: (conversation: Conversation) => void;
  handleUpdateConversation: (
    conversation: Conversation,
    data: KeyValuePair,
  ) => void;
}

const HomeContext = createContext<HomeContextProps>({
  state: initialState,
  dispatch: function (value: ActionType<HomeInitialState>): void {
    throw new Error('Function not implemented.');
  },
  handleNewConversation: function (): void {
    throw new Error('Function not implemented.');
  },
  handleCreateFolder: function (name: string, type: FolderType): void {
    throw new Error('Function not implemented.');
  },
  handleDeleteFolder: function (folderId: string): void {
    throw new Error('Function not implemented.');
  },
  handleUpdateFolder: function (folderId: string, name: string): void {
    throw new Error('Function not implemented.');
  },
  handleSelectConversation: function (conversation: Conversation): void {
    throw new Error('Function not implemented.');
  },
  handleUpdateConversation: function (conversation: Conversation, data: KeyValuePair): void {
    throw new Error('Function not implemented.');
  }
});

export default HomeContext;
