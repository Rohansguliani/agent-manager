import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ChatMessageList, Message } from './ChatMessageList';
import '@testing-library/jest-dom';

describe('ChatMessageList', () => {
  const mockMessages: Message[] = [
    {
      id: '1',
      conversation_id: 'conv1',
      role: 'user',
      content: 'Hello, how are you?',
      created_at: Math.floor(Date.now() / 1000) - 3600,
    },
    {
      id: '2',
      conversation_id: 'conv1',
      role: 'assistant',
      content: 'I am doing well, thank you!',
      created_at: Math.floor(Date.now() / 1000) - 3500,
    },
  ];

  it('renders messages', () => {
    render(
      <ChatMessageList
        messages={mockMessages}
        streamingContent={undefined}
        loading={false}
      />
    );

    expect(screen.getByText('Hello, how are you?')).toBeInTheDocument();
    expect(screen.getByText('I am doing well, thank you!')).toBeInTheDocument();
  });

  it('displays user messages correctly', () => {
    render(
      <ChatMessageList
        messages={[mockMessages[0]]}
        streamingContent={undefined}
        loading={false}
      />
    );

    const userMessage = screen.getByText('Hello, how are you?');
    expect(userMessage).toBeInTheDocument();
  });

  it('displays assistant messages correctly', () => {
    render(
      <ChatMessageList
        messages={[mockMessages[1]]}
        streamingContent={undefined}
        loading={false}
      />
    );

    const assistantMessage = screen.getByText('I am doing well, thank you!');
    expect(assistantMessage).toBeInTheDocument();
  });

  it('shows streaming content', () => {
    render(
      <ChatMessageList
        messages={mockMessages}
        streamingContent="This is streaming..."
        loading={false}
      />
    );

    expect(screen.getByText('This is streaming...')).toBeInTheDocument();
  });

  it('shows loading state', () => {
    render(
      <ChatMessageList
        messages={mockMessages}
        streamingContent={undefined}
        loading={true}
      />
    );

    // Loading state shows "Thinking..." text
    const loadingElement = screen.getByText('Thinking...');
    expect(loadingElement).toBeInTheDocument();
  });

  it('shows empty state when no messages', () => {
    render(
      <ChatMessageList
        messages={[]}
        streamingContent={undefined}
        loading={false}
      />
    );

    expect(screen.getByText('Start a conversation by sending a message')).toBeInTheDocument();
  });

  it('does not show empty state when loading', () => {
    render(
      <ChatMessageList
        messages={[]}
        streamingContent={undefined}
        loading={true}
      />
    );

    expect(screen.queryByText('Start a conversation by sending a message')).not.toBeInTheDocument();
  });

  it('does not show empty state when streaming', () => {
    render(
      <ChatMessageList
        messages={[]}
        streamingContent="Streaming..."
        loading={false}
      />
    );

    expect(screen.queryByText('Start a conversation by sending a message')).not.toBeInTheDocument();
    expect(screen.getByText('Streaming...')).toBeInTheDocument();
  });
});

