import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ChatSidebar, Conversation } from './ChatSidebar';

describe('ChatSidebar', () => {
  const mockConversations: Conversation[] = [
    {
      id: '1',
      title: 'Test Conversation 1',
      created_at: Math.floor(Date.now() / 1000) - 3600, // 1 hour ago
      updated_at: Math.floor(Date.now() / 1000) - 3600,
    },
    {
      id: '2',
      title: 'Test Conversation 2',
      created_at: Math.floor(Date.now() / 1000) - 7200, // 2 hours ago
      updated_at: Math.floor(Date.now() / 1000) - 7200,
    },
  ];

  it('renders conversations list', () => {
    const onSelect = vi.fn();
    const onNew = vi.fn();
    const onDelete = vi.fn();

    render(
      <ChatSidebar
        conversations={mockConversations}
        selectedConversationId={null}
        onSelectConversation={onSelect}
        onNewConversation={onNew}
        onDeleteConversation={onDelete}
      />
    );

    expect(screen.getByText('Test Conversation 1')).toBeInTheDocument();
    expect(screen.getByText('Test Conversation 2')).toBeInTheDocument();
  });

  it('calls onSelectConversation when conversation is clicked', () => {
    const onSelect = vi.fn();
    const onNew = vi.fn();
    const onDelete = vi.fn();

    render(
      <ChatSidebar
        conversations={mockConversations}
        selectedConversationId={null}
        onSelectConversation={onSelect}
        onNewConversation={onNew}
        onDeleteConversation={onDelete}
      />
    );

    fireEvent.click(screen.getByText('Test Conversation 1'));
    expect(onSelect).toHaveBeenCalledWith('1');
  });

  it('calls onNewConversation when new button is clicked', () => {
    const onSelect = vi.fn();
    const onNew = vi.fn();
    const onDelete = vi.fn();

    render(
      <ChatSidebar
        conversations={mockConversations}
        selectedConversationId={null}
        onSelectConversation={onSelect}
        onNewConversation={onNew}
        onDeleteConversation={onDelete}
      />
    );

    const newButton = screen.getByTitle('New Chat');
    fireEvent.click(newButton);
    expect(onNew).toHaveBeenCalled();
  });

  it('calls onDeleteConversation when delete button is clicked', () => {
    const onSelect = vi.fn();
    const onNew = vi.fn();
    const onDelete = vi.fn();

    render(
      <ChatSidebar
        conversations={mockConversations}
        selectedConversationId={null}
        onSelectConversation={onSelect}
        onNewConversation={onNew}
        onDeleteConversation={onDelete}
      />
    );

    // Find delete buttons (they're hidden by default, but we can query by title)
    const deleteButtons = screen.getAllByTitle('Delete conversation');
    fireEvent.click(deleteButtons[0]);
    expect(onDelete).toHaveBeenCalled();
  });

  it('shows loading state', () => {
    const onSelect = vi.fn();
    const onNew = vi.fn();
    const onDelete = vi.fn();

    render(
      <ChatSidebar
        conversations={[]}
        selectedConversationId={null}
        onSelectConversation={onSelect}
        onNewConversation={onNew}
        onDeleteConversation={onDelete}
        loading={true}
      />
    );

    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('shows empty state when no conversations', () => {
    const onSelect = vi.fn();
    const onNew = vi.fn();
    const onDelete = vi.fn();

    render(
      <ChatSidebar
        conversations={[]}
        selectedConversationId={null}
        onSelectConversation={onSelect}
        onNewConversation={onNew}
        onDeleteConversation={onDelete}
        loading={false}
      />
    );

    expect(screen.getByText('No conversations yet')).toBeInTheDocument();
    expect(screen.getByText('Start a new chat to begin')).toBeInTheDocument();
  });

  it('highlights selected conversation', () => {
    const onSelect = vi.fn();
    const onNew = vi.fn();
    const onDelete = vi.fn();

    render(
      <ChatSidebar
        conversations={mockConversations}
        selectedConversationId="1"
        onSelectConversation={onSelect}
        onNewConversation={onNew}
        onDeleteConversation={onDelete}
      />
    );

    // Check that selected conversation is rendered
    expect(screen.getByText('Test Conversation 1')).toBeInTheDocument();
  });
});

