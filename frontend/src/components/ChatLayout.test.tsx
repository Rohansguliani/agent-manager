import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ChatLayout } from './ChatLayout';
import { Conversation, Message } from '../api';
import '@testing-library/jest-dom';

describe('ChatLayout', () => {
  const mockConversations: Conversation[] = [
    {
      id: '1',
      title: 'Test Conversation',
      created_at: Math.floor(Date.now() / 1000),
      updated_at: Math.floor(Date.now() / 1000),
    },
  ];

  const mockMessages: Message[] = [
    {
      id: '1',
      conversation_id: '1',
      role: 'user',
      content: 'Hello',
      created_at: Math.floor(Date.now() / 1000),
    },
  ];

  it('renders sidebar and chat area', () => {
    const onSelect = vi.fn();
    const onNew = vi.fn();
    const onDelete = vi.fn();
    const onSend = vi.fn();

    render(
      <ChatLayout
        conversations={mockConversations}
        selectedConversationId="1"
        messages={mockMessages}
        streamingContent={undefined}
        loading={false}
        onSelectConversation={onSelect}
        onNewConversation={onNew}
        onDeleteConversation={onDelete}
        onSendMessage={onSend}
        sending={false}
      />
    );

    // Check that sidebar is rendered (should have conversations title)
    expect(screen.getByText('Conversations')).toBeInTheDocument();
    
    // Check that messages are rendered
    expect(screen.getByText('Hello')).toBeInTheDocument();
    
    // Check that input is rendered
    expect(screen.getByPlaceholderText('Type your message...')).toBeInTheDocument();
  });

  it('disables input when no conversation is selected', () => {
    const onSelect = vi.fn();
    const onNew = vi.fn();
    const onDelete = vi.fn();
    const onSend = vi.fn();

    render(
      <ChatLayout
        conversations={mockConversations}
        selectedConversationId={null}
        messages={[]}
        streamingContent={undefined}
        loading={false}
        onSelectConversation={onSelect}
        onNewConversation={onNew}
        onDeleteConversation={onDelete}
        onSendMessage={onSend}
        sending={false}
      />
    );

    const input = screen.getByPlaceholderText('Select a conversation or create a new one');
    expect(input).toBeDisabled();
  });

  it('disables input when sending', () => {
    const onSelect = vi.fn();
    const onNew = vi.fn();
    const onDelete = vi.fn();
    const onSend = vi.fn();

    render(
      <ChatLayout
        conversations={mockConversations}
        selectedConversationId="1"
        messages={mockMessages}
        streamingContent={undefined}
        loading={false}
        onSelectConversation={onSelect}
        onNewConversation={onNew}
        onDeleteConversation={onDelete}
        onSendMessage={onSend}
        sending={true}
      />
    );

    const input = screen.getByPlaceholderText('Type your message...');
    expect(input).toBeDisabled();
  });

  it('shows streaming content', () => {
    const onSelect = vi.fn();
    const onNew = vi.fn();
    const onDelete = vi.fn();
    const onSend = vi.fn();

    render(
      <ChatLayout
        conversations={mockConversations}
        selectedConversationId="1"
        messages={mockMessages}
        streamingContent="Streaming response..."
        loading={false}
        onSelectConversation={onSelect}
        onNewConversation={onNew}
        onDeleteConversation={onDelete}
        onSendMessage={onSend}
        sending={true}
      />
    );

    expect(screen.getByText('Streaming response...')).toBeInTheDocument();
  });
});

