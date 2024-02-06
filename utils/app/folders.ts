import { FolderInterface } from '@/types/folder';

export const saveFolders = (folders: FolderInterface[]) => {
  localStorage.setItem('folders', JSON.stringify(folders));
};

export const getFolders = () => {
  let value = localStorage.getItem('folders');
  let folders: FolderInterface[] = value ? JSON.parse(value) : [];
  return folders;
}