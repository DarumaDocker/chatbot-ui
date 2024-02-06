import { Prompt } from '@/types/prompt';

export const updatePrompt = (updatedPrompt: Prompt, allPrompts: Prompt[]) => {
  const updatedPrompts = allPrompts.map((c) => {
    if (c.id === updatedPrompt.id) {
      return updatedPrompt;
    }

    return c;
  });

  savePrompts(updatedPrompts);

  return {
    single: updatedPrompt,
    all: updatedPrompts,
  };
};

export const savePrompts = (prompts: Prompt[]) => {
  localStorage.setItem('prompts', JSON.stringify(prompts));
};

export const getPrompts = () => {
  let prompts: Prompt[] = JSON.parse(localStorage.getItem('prompts') || '[]');
  return prompts;
}

export const saveShowPromptbar = (showPromptbar: boolean) => {
  localStorage.setItem('showPromptbar', JSON.stringify(!showPromptbar));
}

export const getShowPromptbar = () => {
  return localStorage.getItem('showPromptbar') === 'true';
}