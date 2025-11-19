import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { ChatInput } from './ChatInput';
import '@testing-library/jest-dom';

describe('ChatInput', () => {
  it('renders textarea and send button', () => {
    const onSend = vi.fn();

    render(<ChatInput onSend={onSend} />);

    expect(screen.getByPlaceholderText('Type your message...')).toBeInTheDocument();
    expect(screen.getByTitle('Send message (Enter)')).toBeInTheDocument();
  });

  it('calls onSend when form is submitted', async () => {
    const onSend = vi.fn();

    render(<ChatInput onSend={onSend} />);

    const textarea = screen.getByPlaceholderText('Type your message...');
    const form = textarea.closest('form');

    fireEvent.change(textarea, { target: { value: 'Test message' } });
    fireEvent.submit(form!);

    await waitFor(() => {
      expect(onSend).toHaveBeenCalledWith('Test message');
    });
  });

  it('calls onSend when Enter is pressed', async () => {
    const onSend = vi.fn();

    render(<ChatInput onSend={onSend} />);

    const textarea = screen.getByPlaceholderText('Type your message...');

    fireEvent.change(textarea, { target: { value: 'Test message' } });
    fireEvent.keyDown(textarea, { key: 'Enter', shiftKey: false });

    await waitFor(() => {
      expect(onSend).toHaveBeenCalledWith('Test message');
    });
  });

  it('does not call onSend when Shift+Enter is pressed', () => {
    const onSend = vi.fn();

    render(<ChatInput onSend={onSend} />);

    const textarea = screen.getByPlaceholderText('Type your message...');

    fireEvent.change(textarea, { target: { value: 'Test message' } });
    fireEvent.keyDown(textarea, { key: 'Enter', shiftKey: true });

    expect(onSend).not.toHaveBeenCalled();
  });

  it('disables input when disabled prop is true', () => {
    const onSend = vi.fn();

    render(<ChatInput onSend={onSend} disabled={true} />);

    const textarea = screen.getByPlaceholderText('Type your message...');
    const button = screen.getByTitle('Send message (Enter)');

    expect(textarea).toBeDisabled();
    expect(button).toBeDisabled();
  });

  it('disables send button when message is empty', () => {
    const onSend = vi.fn();

    render(<ChatInput onSend={onSend} />);

    const button = screen.getByTitle('Send message (Enter)');
    expect(button).toBeDisabled();
  });

  it('enables send button when message has content', () => {
    const onSend = vi.fn();

    render(<ChatInput onSend={onSend} />);

    const textarea = screen.getByPlaceholderText('Type your message...');
    const button = screen.getByTitle('Send message (Enter)');

    fireEvent.change(textarea, { target: { value: 'Test' } });

    expect(button).not.toBeDisabled();
  });

  it('clears input after sending', async () => {
    const onSend = vi.fn();

    render(<ChatInput onSend={onSend} />);

    const textarea = screen.getByPlaceholderText('Type your message...') as HTMLTextAreaElement;
    const form = textarea.closest('form');

    fireEvent.change(textarea, { target: { value: 'Test message' } });
    fireEvent.submit(form!);

    await waitFor(() => {
      expect(textarea.value).toBe('');
    });
  });

  it('uses custom placeholder', () => {
    const onSend = vi.fn();

    render(<ChatInput onSend={onSend} placeholder="Custom placeholder" />);

    expect(screen.getByPlaceholderText('Custom placeholder')).toBeInTheDocument();
  });
});

