import { useState, useRef, useEffect, useCallback } from 'react';
import { api } from '../api';
import toast from 'react-hot-toast';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';

interface Message {
  role: 'user' | 'assistant';
  content: string;
  id: string;
  images?: string[]; // Array of image URLs (data URLs for preview)
}

interface Conversation {
  id: string;
  title: string;
  messages: Message[];
  createdAt: number;
  updatedAt: number;
}

// Obsidian black theme colors
const obsidianTheme = {
  black: '#000000',
  gray900: '#0a0a0a',
  gray800: '#1a1a1a',
  gray700: '#2a2a2a',
  gray600: '#3a3a3a',
  gray500: '#4a4a4a',
  white: '#ffffff',
  textPrimary: '#e0e0e0',
  textSecondary: '#a0a0a0',
  textTertiary: '#606060',
  accent: '#4a9eff',
  accentHover: '#5aaeff',
  userMessage: '#4a9eff',
  assistantMessage: '#1a1a1a',
  border: '#2a2a2a',
};

export function SimpleChat() {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [selectedConversationId, setSelectedConversationId] = useState<string | null>(null);
  const [input, setInput] = useState('');
  const [loadingConversationId, setLoadingConversationId] = useState<string | null>(null); // Track which conversation is loading
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [selectedImages, setSelectedImages] = useState<File[]>([]);
  const [imagePreviews, setImagePreviews] = useState<string[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>('gemini-2.5-flash');
  const [modelMenuOpen, setModelMenuOpen] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const modelMenuRef = useRef<HTMLDivElement>(null);

  // Get current conversation
  const currentConversation = conversations.find(c => c.id === selectedConversationId);
  const messages = currentConversation?.messages || [];
  // Only show loading indicator if there's a selected conversation AND it's the one loading
  const isCurrentConversationLoading = selectedConversationId !== null && selectedConversationId === loadingConversationId;

  // Load conversations from database on mount
  useEffect(() => {
    // Clear any stale loading state on mount
    setLoadingConversationId(null);
    
    const loadConversations = async () => {
      try {
        const dbConversations = await api.listConversations();
        // Convert database conversations to local format
        const localConversations: Conversation[] = dbConversations.map((conv) => ({
          id: conv.id,
          title: conv.title,
          messages: [], // Messages loaded separately when conversation is selected
          createdAt: conv.created_at * 1000, // Convert to milliseconds
          updatedAt: conv.updated_at * 1000,
        }));
        setConversations(localConversations);
      } catch (error) {
        console.error('Failed to load conversations:', error);
        // Don't show error toast on initial load - just log it
      }
    };
    loadConversations();
  }, []);

  // Load messages when a conversation is selected
  useEffect(() => {
    if (!selectedConversationId) return;

    const loadMessages = async () => {
      try {
        const conversationData = await api.getConversation(selectedConversationId);
        // Update conversation with messages
        setConversations((prev) => {
          // Check if conversation still exists and messages aren't already loaded
          const conv = prev.find((c) => c.id === selectedConversationId);
          if (!conv || conv.messages.length > 0) {
            return prev; // Already loaded or conversation doesn't exist
          }
          
          return prev.map((conv) =>
            conv.id === selectedConversationId
              ? {
                  ...conv,
                  messages: conversationData.messages.map((msg) => ({
                    id: msg.id,
                    role: msg.role as 'user' | 'assistant',
                    content: msg.content,
                    images: undefined, // Images not stored in DB for now
                  })),
                }
              : conv
          );
        });
      } catch (error) {
        console.error('Failed to load messages:', error);
        toast.error('Failed to load conversation messages');
      }
    };

    // Check if messages need to be loaded
    const conv = conversations.find((c) => c.id === selectedConversationId);
    if (conv && conv.messages.length === 0) {
      loadMessages();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedConversationId]); // Only depend on selectedConversationId to avoid infinite loops

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, isCurrentConversationLoading]);

  // Auto-resize textarea
  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.style.height = 'auto';
      inputRef.current.style.height = `${Math.min(inputRef.current.scrollHeight, 200)}px`;
    }
  }, [input]);

  // Close model menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (modelMenuRef.current && !modelMenuRef.current.contains(event.target as Node)) {
        setModelMenuOpen(false);
      }
    };

    if (modelMenuOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => {
        document.removeEventListener('mousedown', handleClickOutside);
      };
    }
  }, [modelMenuOpen]);

  // Available models
  const availableModels = [
    { value: 'gemini-2.5-flash', label: 'Gemini 2.5 Flash', description: 'Fast responses' },
    { value: 'gemini-2.5-pro', label: 'Gemini 2.5 Pro', description: 'Better reasoning' },
    { value: '', label: 'Default', description: 'CLI default model' },
  ];

  // Handle image selection
  const handleImageSelect = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || []);
    if (files.length === 0) return;

    // Validate file types and sizes
    const validFiles: File[] = [];
    const previewPromises: Promise<string>[] = [];

    files.forEach((file) => {
      // Check file type
      if (!file.type.match(/^image\/(png|jpeg|jpg|webp|heic)$/i)) {
        toast.error(`Invalid file type: ${file.name}. Supported: PNG, JPEG, WEBP, HEIC`);
        return;
      }

      // Check file size (7MB limit)
      if (file.size > 7 * 1024 * 1024) {
        toast.error(`File too large: ${file.name}. Maximum size: 7MB`);
        return;
      }

      validFiles.push(file);
      
      // Create preview promise
      const previewPromise = new Promise<string>((resolve) => {
        const reader = new FileReader();
        reader.onload = (e) => {
          if (e.target?.result) {
            resolve(e.target.result as string);
          }
        };
        reader.readAsDataURL(file);
      });
      previewPromises.push(previewPromise);
    });

    // Update selected images immediately
    setSelectedImages(prev => [...prev, ...validFiles]);

    // Update previews when all are loaded
    Promise.all(previewPromises).then((previews) => {
      setImagePreviews(prev => [...prev, ...previews]);
    });
  }, []);

  // Remove image
  const handleRemoveImage = useCallback((index: number) => {
    setSelectedImages(prev => prev.filter((_, i) => i !== index));
    setImagePreviews(prev => prev.filter((_, i) => i !== index));
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  }, []);

  // Generate conversation title from first message
  const generateTitle = (message: string): string => {
    const trimmed = message.trim();
    if (trimmed.length <= 50) return trimmed;
    return trimmed.substring(0, 47) + '...';
  };

  const handleSend = useCallback(async () => {
    const message = input.trim();
    // Allow sending if:
    // 1. There's a message
    // 2. Either no conversation is selected (new chat), OR the selected conversation is not currently loading
    if (!message) return;
    if (selectedConversationId && loadingConversationId === selectedConversationId) {
      // Can't send to a conversation that's already loading
      return;
    }

    let conversationId = selectedConversationId;

    // Create new conversation if none selected
    // Note: Backend will create the conversation in SQLite when the first message is sent
    if (!conversationId) {
      conversationId = `conv-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
      
      // Create conversation in local state (backend will create it in DB)
      const newConv: Conversation = {
        id: conversationId,
        title: generateTitle(message),
        messages: [],
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };
      setConversations(prev => [newConv, ...prev]);
      setSelectedConversationId(conversationId);
    }

    // Add user message optimistically
    const userMessage: Message = {
      id: `msg-${Date.now()}-user`,
      role: 'user',
      content: message,
      images: imagePreviews.length > 0 ? [...imagePreviews] : undefined,
    };

    // Store conversationId in const before async operations (needed for finally block)
    const currentConversationId = conversationId;
    
    // Update conversation with user message
    setConversations(prev => prev.map(conv => 
      conv.id === currentConversationId
        ? {
            ...conv,
            messages: [...conv.messages, userMessage],
            updatedAt: Date.now(),
            title: conv.messages.length === 0 ? generateTitle(message) : conv.title,
          }
        : conv
    ));

    const imagesToSend = [...selectedImages];
    setInput('');
    setSelectedImages([]);
    setImagePreviews([]);
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
    // Set loading state for this specific conversation
    setLoadingConversationId(currentConversationId);

    try {
      const modelToUse = selectedModel || undefined;
      const response = imagesToSend.length > 0
        ? await api.simpleChatWithImages(message, imagesToSend, currentConversationId || undefined, modelToUse)
        : await api.simpleChat(message, currentConversationId || undefined, modelToUse);
      
      // Update conversation with assistant response
      const assistantMessage: Message = {
        id: `msg-${Date.now()}-assistant`,
        role: 'assistant',
        content: response.response,
      };

      setConversations(prev => prev.map(conv => 
        conv.id === response.conversation_id
          ? {
              ...conv,
              messages: [...conv.messages, assistantMessage],
              updatedAt: Date.now(),
            }
          : conv
      ));
      
      // Reload conversations from database to get updated titles/timestamps
      // (This ensures we're in sync with the database)
      try {
        const dbConversations = await api.listConversations();
        setConversations(prev => {
          const updated = prev.map(localConv => {
            const dbConv = dbConversations.find(c => c.id === localConv.id);
            if (dbConv) {
              return {
                ...localConv,
                title: dbConv.title,
                updatedAt: dbConv.updated_at * 1000,
              };
            }
            return localConv;
          });
          // Add any new conversations from DB that aren't in local state
          const localIds = new Set(prev.map(c => c.id));
          const newConvs = dbConversations
            .filter(c => !localIds.has(c.id))
            .map(conv => ({
              id: conv.id,
              title: conv.title,
              messages: [],
              createdAt: conv.created_at * 1000,
              updatedAt: conv.updated_at * 1000,
            }));
          return [...updated, ...newConvs].sort((a, b) => b.updatedAt - a.updatedAt);
        });
      } catch (error) {
        console.error('Failed to refresh conversations:', error);
        // Non-critical - continue with local state
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Failed to get response';
      toast.error(errorMsg);
      console.error('Chat error:', error);
      
      // Remove user message on error
      setConversations(prev => prev.map(conv => 
        conv.id === currentConversationId
          ? {
              ...conv,
              messages: conv.messages.filter(m => m.id !== userMessage.id),
            }
          : conv
      ));
    } finally {
      // Clear loading state for this conversation
      setLoadingConversationId(prev => prev === currentConversationId ? null : prev);
    }
  }, [input, loadingConversationId, selectedConversationId, selectedImages, imagePreviews, selectedModel]);

  const handleNewConversation = useCallback(() => {
    setSelectedConversationId(null);
    setInput('');
    // Don't clear loading state - other conversations can still be loading
    // But clear it if we're switching away from a loading conversation
    if (inputRef.current) {
      inputRef.current.focus();
    }
  }, []);

  const handleSelectConversation = useCallback((id: string) => {
    setSelectedConversationId(id);
  }, []);

  const handleDeleteConversation = useCallback(async (id: string) => {
    try {
      await api.deleteConversation(id);
      setConversations(prev => prev.filter(conv => conv.id !== id));
      if (selectedConversationId === id) {
        setSelectedConversationId(null);
      }
      toast.success('Conversation deleted');
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Failed to delete conversation';
      toast.error(errorMsg);
    }
  }, [selectedConversationId]);

  const handleKeyPress = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const formatTime = (timestamp: number): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  };

  return (
    <div style={styles.container}>
      {/* Sidebar */}
      <aside style={{
        ...styles.sidebar,
        width: sidebarOpen ? '280px' : '0',
        borderRight: sidebarOpen ? `1px solid ${obsidianTheme.border}` : 'none',
      }}>
        <div style={styles.sidebarHeader}>
          <h2 style={styles.sidebarTitle}>Chats</h2>
          <button
            onClick={handleNewConversation}
            className="new-chat-button"
            style={styles.newChatButton}
            title="New Chat"
            aria-label="New Chat"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="12" y1="5" x2="12" y2="19" />
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
          </button>
        </div>

        <div style={styles.conversationList}>
          {conversations.length === 0 ? (
            <div style={styles.emptyState}>
              <p style={styles.emptyText}>No conversations yet</p>
              <p style={styles.emptySubtext}>Start chatting to create one</p>
            </div>
          ) : (
            conversations.map((conv) => (
              <div
                key={conv.id}
                className="conversation-item"
                style={{
                  ...styles.conversationItem,
                  ...(selectedConversationId === conv.id ? styles.conversationItemSelected : {}),
                }}
                onClick={() => handleSelectConversation(conv.id)}
              >
                <div style={styles.conversationContent}>
                  <div style={styles.conversationTitle}>{conv.title}</div>
                  <div style={styles.conversationTime}>{formatTime(conv.updatedAt)}</div>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDeleteConversation(conv.id);
                  }}
                  className="delete-button"
                  style={styles.deleteButton}
                  title="Delete"
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                  </svg>
                </button>
              </div>
            ))
          )}
        </div>
      </aside>

      {/* Main Chat Area */}
      <div style={styles.chatArea}>
        {/* Messages */}
        <div style={styles.messagesContainer}>
          {messages.length === 0 && !isCurrentConversationLoading && (
            <div style={styles.welcomeMessage}>
              <div style={styles.welcomeIcon}>💬</div>
              <h1 style={styles.welcomeTitle}>Start a conversation</h1>
              <p style={styles.welcomeText}>Type a message below to begin chatting</p>
            </div>
          )}

          {messages.map((msg) => (
            <div
              key={msg.id}
              style={{
                ...styles.message,
                ...(msg.role === 'user' ? styles.userMessage : styles.assistantMessage),
              }}
            >
              {msg.images && msg.images.length > 0 && (
                <div style={styles.messageImages}>
                  {msg.images.map((img, idx) => (
                    <img
                      key={idx}
                      src={img}
                      alt={`Upload ${idx + 1}`}
                      style={styles.messageImage}
                    />
                  ))}
                </div>
              )}
              <div style={styles.messageContent}>
                {msg.role === 'assistant' ? (
                  <div style={styles.markdownWrapper}>
                    <ReactMarkdown
                    remarkPlugins={[remarkGfm]}
                    components={{
                      // Headings
                      h1: ({ node, ...props }) => {
                        const isFirst = node?.position?.start.line === 1;
                        return <h1 style={{ ...styles.markdownH1, marginTop: isFirst ? '0px' : styles.markdownH1.marginTop }} {...props} />;
                      },
                      h2: ({ node, ...props }) => {
                        const isFirst = node?.position?.start.line === 1;
                        return <h2 style={{ ...styles.markdownH2, marginTop: isFirst ? '0px' : styles.markdownH2.marginTop }} {...props} />;
                      },
                      h3: ({ node, ...props }) => {
                        const isFirst = node?.position?.start.line === 1;
                        return <h3 style={{ ...styles.markdownH3, marginTop: isFirst ? '0px' : styles.markdownH3.marginTop }} {...props} />;
                      },
                      // Paragraphs
                      p: ({ node, ...props }) => {
                        const isFirst = node?.position?.start.line === 1;
                        return <p style={{ ...styles.markdownP, marginTop: isFirst ? '0px' : styles.markdownP.marginTop }} {...props} />;
                      },
                      // Lists
                      ul: ({ node, ...props }) => <ul style={styles.markdownUl} {...props} />,
                      ol: ({ node, ...props }) => <ol style={styles.markdownOl} {...props} />,
                      li: ({ node, ...props }) => <li style={styles.markdownLi} {...props} />,
                      // Bold and italic
                      strong: ({ node, ...props }) => <strong style={styles.markdownStrong} {...props} />,
                      em: ({ node, ...props }) => <em style={styles.markdownEm} {...props} />,
                      // Code blocks
                      code: ({ node, inline, className, children, ...props }: any) => {
                        const match = /language-(\w+)/.exec(className || '');
                        const language = match ? match[1] : '';
                        return !inline && match ? (
                          <SyntaxHighlighter
                            style={vscDarkPlus}
                            language={language}
                            PreTag="div"
                            customStyle={styles.codeBlock}
                            {...props}
                          >
                            {String(children).replace(/\n$/, '')}
                          </SyntaxHighlighter>
                        ) : (
                          <code style={styles.inlineCode} {...props}>
                            {children}
                          </code>
                        );
                      },
                      // Blockquotes
                      blockquote: ({ node, ...props }) => <blockquote style={styles.markdownBlockquote} {...props} />,
                      // Links
                      a: ({ node, ...props }) => <a style={styles.markdownLink} target="_blank" rel="noopener noreferrer" {...props} />,
                      // Horizontal rule
                      hr: ({ node, ...props }) => <hr style={styles.markdownHr} {...props} />,
                      // Tables (from remark-gfm)
                      table: ({ node, ...props }) => <table style={styles.markdownTable} {...props} />,
                      thead: ({ node, ...props }) => <thead style={styles.markdownThead} {...props} />,
                      tbody: ({ node, ...props }) => <tbody {...props} />,
                      tr: ({ node, ...props }) => <tr style={styles.markdownTr} {...props} />,
                      th: ({ node, ...props }) => <th style={styles.markdownTh} {...props} />,
                      td: ({ node, ...props }) => <td style={styles.markdownTd} {...props} />,
                    }}
                  >
                    {msg.content}
                  </ReactMarkdown>
                  </div>
                ) : (
                  msg.content
                )}
              </div>
            </div>
          ))}

          {isCurrentConversationLoading && (
            <div style={{ ...styles.message, ...styles.assistantMessage }}>
              <div className="typing-indicator" style={styles.typingIndicator}>
                <span></span>
                <span></span>
                <span></span>
              </div>
            </div>
          )}

          <div ref={messagesEndRef} />
        </div>

        {/* Input Area */}
        <div style={styles.inputContainer}>
          {/* Model Selector */}
          <div style={styles.modelSelectorContainer}>
            <div style={{ position: 'relative' }} ref={modelMenuRef}>
              <button
                onClick={() => setModelMenuOpen(!modelMenuOpen)}
                className="model-selector-button"
                style={styles.modelSelectorButton}
                title="Select model"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginRight: '8px' }}>
                  <rect x="3" y="3" width="18" height="18" rx="2" />
                  <path d="M9 9h6M9 15h6" />
                </svg>
                <span style={styles.modelSelectorText}>
                  {availableModels.find(m => m.value === selectedModel)?.label || 'Select Model'}
                </span>
                <svg 
                  width="14" 
                  height="14" 
                  viewBox="0 0 24 24" 
                  fill="none" 
                  stroke="currentColor" 
                  strokeWidth="2"
                  style={{
                    marginLeft: '8px',
                    transform: modelMenuOpen ? 'rotate(180deg)' : 'rotate(0deg)',
                    transition: 'transform 0.2s ease',
                  }}
                >
                  <polyline points="6 9 12 15 18 9" />
                </svg>
              </button>
              
              {modelMenuOpen && (
                <div style={styles.modelMenu}>
                  {availableModels.map((model) => (
                    <button
                      key={model.value}
                      onClick={() => {
                        setSelectedModel(model.value);
                        setModelMenuOpen(false);
                      }}
                      style={{
                        ...styles.modelMenuItem,
                        ...(selectedModel === model.value ? styles.modelMenuItemSelected : {}),
                      }}
                    >
                      <div style={styles.modelMenuItemContent}>
                        <div style={styles.modelMenuItemLabel}>{model.label}</div>
                        <div style={styles.modelMenuItemDescription}>{model.description}</div>
                      </div>
                      {selectedModel === model.value && (
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                          <polyline points="20 6 9 17 4 12" />
                        </svg>
                      )}
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Image Previews */}
          {imagePreviews.length > 0 && (
            <div style={styles.imagePreviewContainer}>
              {imagePreviews.map((preview, idx) => (
                <div key={idx} style={styles.imagePreviewWrapper}>
                  <img src={preview} alt={`Preview ${idx + 1}`} style={styles.imagePreview} />
                  <button
                    onClick={() => handleRemoveImage(idx)}
                    style={styles.imagePreviewRemove}
                    title="Remove image"
                  >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <line x1="18" y1="6" x2="6" y2="18" />
                      <line x1="6" y1="6" x2="18" y2="18" />
                    </svg>
                  </button>
                </div>
              ))}
            </div>
          )}
          
          <div style={styles.inputWrapper}>
            <input
              ref={fileInputRef}
              type="file"
              accept="image/png,image/jpeg,image/jpg,image/webp,image/heic"
              multiple
              onChange={handleImageSelect}
              style={{ display: 'none' }}
            />
            <button
              onClick={() => fileInputRef.current?.click()}
              className="image-upload-button"
              disabled={isCurrentConversationLoading}
              style={styles.imageUploadButton}
              title="Upload images"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                <circle cx="8.5" cy="8.5" r="1.5" />
                <polyline points="21 15 16 10 5 21" />
              </svg>
            </button>
            <textarea
              ref={inputRef}
              className="textarea"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder={selectedConversationId ? "Type your message..." : "Type a message to start a new conversation..."}
              disabled={isCurrentConversationLoading}
              style={styles.textarea}
              rows={1}
            />
            <button
              onClick={handleSend}
              className="send-button"
              disabled={isCurrentConversationLoading || (!input.trim() && selectedImages.length === 0)}
              style={{
                ...styles.sendButton,
                ...((isCurrentConversationLoading || (!input.trim() && selectedImages.length === 0)) ? styles.sendButtonDisabled : {}),
              }}
              title="Send (Enter)"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <line x1="22" y1="2" x2="11" y2="13" />
                <polygon points="22 2 15 22 11 13 2 9 22 2" />
              </svg>
            </button>
          </div>
        </div>
      </div>

      {/* Sidebar Toggle */}
      <button
        onClick={() => setSidebarOpen(!sidebarOpen)}
        style={{
          ...styles.sidebarToggle,
          left: sidebarOpen ? '296px' : '16px',
        }}
        title={sidebarOpen ? "Hide sidebar" : "Show sidebar"}
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          {sidebarOpen ? (
            <>
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </>
          ) : (
            <>
              <line x1="3" y1="12" x2="21" y2="12" />
              <line x1="3" y1="6" x2="21" y2="6" />
              <line x1="3" y1="18" x2="21" y2="18" />
            </>
          )}
        </svg>
      </button>
    </div>
  );
}

const styles = {
  container: {
    display: 'flex',
    height: '100vh',
    width: '100vw',
    backgroundColor: obsidianTheme.black,
    color: obsidianTheme.textPrimary,
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    overflow: 'hidden',
  } as React.CSSProperties,

  sidebar: {
    display: 'flex',
    flexDirection: 'column' as const,
    backgroundColor: obsidianTheme.gray900,
    transition: 'width 0.3s ease',
    overflow: 'hidden',
  } as React.CSSProperties,

  sidebarHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '16px',
    borderBottom: `1px solid ${obsidianTheme.border}`,
  } as React.CSSProperties,

  sidebarTitle: {
    margin: 0,
    fontSize: '18px',
    fontWeight: 600,
    color: obsidianTheme.textPrimary,
  } as React.CSSProperties,

  newChatButton: {
    width: '36px',
    height: '36px',
    borderRadius: '10px',
    border: `1px solid ${obsidianTheme.border}`,
    backgroundColor: obsidianTheme.gray800,
    color: obsidianTheme.textPrimary,
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: 'all 0.2s ease',
  } as React.CSSProperties,

  conversationList: {
    flex: 1,
    overflowY: 'auto' as const,
    padding: '8px',
  } as React.CSSProperties,

  conversationItem: {
    padding: '12px',
    marginBottom: '4px',
    borderRadius: '12px',
    cursor: 'pointer',
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    transition: 'all 0.2s ease',
    backgroundColor: 'transparent',
    position: 'relative' as const,
  } as React.CSSProperties,

  conversationItemSelected: {
    backgroundColor: obsidianTheme.gray800,
  } as React.CSSProperties,

  conversationContent: {
    flex: 1,
    minWidth: 0,
  } as React.CSSProperties,

  conversationTitle: {
    fontSize: '14px',
    fontWeight: 500,
    color: obsidianTheme.textPrimary,
    whiteSpace: 'nowrap' as const,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    marginBottom: '4px',
  } as React.CSSProperties,

  conversationTime: {
    fontSize: '12px',
    color: obsidianTheme.textTertiary,
  } as React.CSSProperties,

  deleteButton: {
    width: '24px',
    height: '24px',
    borderRadius: '6px',
    border: 'none',
    backgroundColor: 'transparent',
    color: obsidianTheme.textTertiary,
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    opacity: 0,
    transition: 'all 0.2s ease',
    padding: 0,
  } as React.CSSProperties,

  emptyState: {
    padding: '40px 20px',
    textAlign: 'center' as const,
    color: obsidianTheme.textTertiary,
  } as React.CSSProperties,

  emptyText: {
    fontSize: '14px',
    margin: '0 0 4px 0',
    color: obsidianTheme.textSecondary,
  } as React.CSSProperties,

  emptySubtext: {
    fontSize: '12px',
    margin: 0,
    color: obsidianTheme.textTertiary,
  } as React.CSSProperties,

  chatArea: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column' as const,
    overflow: 'hidden',
    backgroundColor: obsidianTheme.black,
  } as React.CSSProperties,

  messagesContainer: {
    flex: 1,
    overflowY: 'auto' as const,
    padding: '24px',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '16px',
  } as React.CSSProperties,

  welcomeMessage: {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    textAlign: 'center' as const,
    color: obsidianTheme.textSecondary,
  } as React.CSSProperties,

  welcomeIcon: {
    fontSize: '64px',
    marginBottom: '16px',
  } as React.CSSProperties,

  welcomeTitle: {
    fontSize: '24px',
    fontWeight: 600,
    margin: '0 0 8px 0',
    color: obsidianTheme.textPrimary,
  } as React.CSSProperties,

  welcomeText: {
    fontSize: '16px',
    margin: 0,
    color: obsidianTheme.textSecondary,
  } as React.CSSProperties,

  message: {
    maxWidth: '80%',
    padding: '16px 20px',
    borderRadius: '20px',
    wordWrap: 'break-word' as const,
    whiteSpace: 'pre-wrap' as const,
    lineHeight: 1.6,
    transition: 'all 0.2s ease',
  } as React.CSSProperties,

  userMessage: {
    alignSelf: 'flex-end' as const,
    backgroundColor: obsidianTheme.userMessage,
    color: obsidianTheme.white,
    borderBottomRightRadius: '4px',
  } as React.CSSProperties,

  assistantMessage: {
    alignSelf: 'flex-start' as const,
    backgroundColor: obsidianTheme.assistantMessage,
    color: obsidianTheme.textPrimary,
    border: `1px solid ${obsidianTheme.border}`,
    borderBottomLeftRadius: '4px',
  } as React.CSSProperties,

  messageContent: {
    fontSize: '15px',
    lineHeight: 1.6,
  } as React.CSSProperties,

  markdownWrapper: {
    // First and last child margins handled via component styles
  } as React.CSSProperties,

  // Markdown styles
  markdownH1: {
    fontSize: '24px',
    fontWeight: 600,
    marginTop: '16px',
    marginBottom: '12px',
    color: obsidianTheme.textPrimary,
    lineHeight: 1.3,
  } as React.CSSProperties,

  markdownH2: {
    fontSize: '20px',
    fontWeight: 600,
    marginTop: '14px',
    marginBottom: '10px',
    color: obsidianTheme.textPrimary,
    lineHeight: 1.3,
  } as React.CSSProperties,

  markdownH3: {
    fontSize: '18px',
    fontWeight: 600,
    marginTop: '12px',
    marginBottom: '8px',
    color: obsidianTheme.textPrimary,
    lineHeight: 1.3,
  } as React.CSSProperties,

  markdownP: {
    marginTop: '8px',
    marginBottom: '8px',
    color: obsidianTheme.textPrimary,
  } as React.CSSProperties,

  markdownUl: {
    marginTop: '8px',
    marginBottom: '8px',
    paddingLeft: '24px',
    color: obsidianTheme.textPrimary,
  } as React.CSSProperties,

  markdownOl: {
    marginTop: '8px',
    marginBottom: '8px',
    paddingLeft: '24px',
    color: obsidianTheme.textPrimary,
  } as React.CSSProperties,

  markdownLi: {
    marginTop: '4px',
    marginBottom: '4px',
    color: obsidianTheme.textPrimary,
  } as React.CSSProperties,

  markdownStrong: {
    fontWeight: 600,
    color: obsidianTheme.textPrimary,
  } as React.CSSProperties,

  markdownEm: {
    fontStyle: 'italic',
    color: obsidianTheme.textPrimary,
  } as React.CSSProperties,

  inlineCode: {
    backgroundColor: obsidianTheme.gray800,
    color: obsidianTheme.accent,
    padding: '2px 6px',
    borderRadius: '4px',
    fontSize: '14px',
    fontFamily: 'Monaco, "Courier New", monospace',
    border: `1px solid ${obsidianTheme.border}`,
  } as React.CSSProperties,

  codeBlock: {
    marginTop: '12px',
    marginBottom: '12px',
    borderRadius: '8px',
    padding: '16px',
    fontSize: '14px',
    fontFamily: 'Monaco, "Courier New", monospace',
    overflowX: 'auto' as const,
    border: `1px solid ${obsidianTheme.border}`,
  } as React.CSSProperties,

  markdownBlockquote: {
    borderLeft: `4px solid ${obsidianTheme.accent}`,
    paddingLeft: '16px',
    marginTop: '12px',
    marginBottom: '12px',
    marginLeft: 0,
    marginRight: 0,
    color: obsidianTheme.textSecondary,
    fontStyle: 'italic',
  } as React.CSSProperties,

  markdownLink: {
    color: obsidianTheme.accent,
    textDecoration: 'underline',
    textDecorationColor: obsidianTheme.accent + '80',
  } as React.CSSProperties,

  markdownHr: {
    border: 'none',
    borderTop: `1px solid ${obsidianTheme.border}`,
    marginTop: '16px',
    marginBottom: '16px',
  } as React.CSSProperties,

  markdownTable: {
    width: '100%',
    borderCollapse: 'collapse' as const,
    marginTop: '12px',
    marginBottom: '12px',
    fontSize: '14px',
  } as React.CSSProperties,

  markdownThead: {
    backgroundColor: obsidianTheme.gray800,
  } as React.CSSProperties,

  markdownTr: {
    borderBottom: `1px solid ${obsidianTheme.border}`,
  } as React.CSSProperties,

  markdownTh: {
    padding: '8px 12px',
    textAlign: 'left' as const,
    fontWeight: 600,
    color: obsidianTheme.textPrimary,
  } as React.CSSProperties,

  markdownTd: {
    padding: '8px 12px',
    color: obsidianTheme.textPrimary,
  } as React.CSSProperties,

  messageImages: {
    display: 'flex',
    flexWrap: 'wrap' as const,
    gap: '8px',
    marginBottom: '12px',
  } as React.CSSProperties,

  messageImage: {
    maxWidth: '200px',
    maxHeight: '200px',
    borderRadius: '8px',
    objectFit: 'cover' as const,
    border: `1px solid ${obsidianTheme.border}`,
  } as React.CSSProperties,

  imagePreviewContainer: {
    display: 'flex',
    flexWrap: 'wrap' as const,
    gap: '8px',
    padding: '12px 24px 0',
  } as React.CSSProperties,

  imagePreviewWrapper: {
    position: 'relative' as const,
    display: 'inline-block',
  } as React.CSSProperties,

  imagePreview: {
    width: '80px',
    height: '80px',
    borderRadius: '8px',
    objectFit: 'cover' as const,
    border: `1px solid ${obsidianTheme.border}`,
  } as React.CSSProperties,

  imagePreviewRemove: {
    position: 'absolute' as const,
    top: '-8px',
    right: '-8px',
    width: '24px',
    height: '24px',
    borderRadius: '50%',
    border: `1px solid ${obsidianTheme.border}`,
    backgroundColor: obsidianTheme.gray800,
    color: obsidianTheme.textPrimary,
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 0,
  } as React.CSSProperties,

  imageUploadButton: {
    width: '44px',
    height: '44px',
    borderRadius: '22px',
    border: `1px solid ${obsidianTheme.border}`,
    backgroundColor: obsidianTheme.gray800,
    color: obsidianTheme.textPrimary,
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: 'all 0.2s ease',
    flexShrink: 0,
  } as React.CSSProperties,

  typingIndicator: {
    display: 'flex',
    gap: '6px',
    padding: '8px 0',
  } as React.CSSProperties,

  inputContainer: {
    padding: '20px 24px',
    borderTop: `1px solid ${obsidianTheme.border}`,
    backgroundColor: obsidianTheme.gray900,
  } as React.CSSProperties,

  modelSelectorContainer: {
    marginBottom: '12px',
    display: 'flex',
    justifyContent: 'flex-start',
  } as React.CSSProperties,

  modelSelectorButton: {
    display: 'flex',
    alignItems: 'center',
    padding: '8px 12px',
    borderRadius: '8px',
    border: `1px solid ${obsidianTheme.border}`,
    backgroundColor: obsidianTheme.gray800,
    color: obsidianTheme.textPrimary,
    cursor: 'pointer',
    transition: 'all 0.2s ease',
    fontSize: '13px',
    fontFamily: 'inherit',
  } as React.CSSProperties,

  modelSelectorText: {
    color: obsidianTheme.textPrimary,
    fontWeight: 500,
  } as React.CSSProperties,

  modelMenu: {
    position: 'absolute' as const,
    bottom: '100%',
    left: 0,
    marginBottom: '8px',
    minWidth: '220px',
    backgroundColor: obsidianTheme.gray800,
    border: `1px solid ${obsidianTheme.border}`,
    borderRadius: '12px',
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)',
    zIndex: 1000,
    overflow: 'hidden',
  } as React.CSSProperties,

  modelMenuItem: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '12px 16px',
    border: 'none',
    backgroundColor: 'transparent',
    color: obsidianTheme.textPrimary,
    cursor: 'pointer',
    transition: 'all 0.2s ease',
    width: '100%',
    textAlign: 'left' as const,
  } as React.CSSProperties,

  modelMenuItemSelected: {
    backgroundColor: obsidianTheme.gray700,
  } as React.CSSProperties,

  modelMenuItemContent: {
    flex: 1,
  } as React.CSSProperties,

  modelMenuItemLabel: {
    fontSize: '14px',
    fontWeight: 500,
    color: obsidianTheme.textPrimary,
    marginBottom: '2px',
  } as React.CSSProperties,

  modelMenuItemDescription: {
    fontSize: '12px',
    color: obsidianTheme.textSecondary,
  } as React.CSSProperties,

  inputWrapper: {
    display: 'flex',
    gap: '12px',
    alignItems: 'flex-end',
    maxWidth: '100%',
  } as React.CSSProperties,

  textarea: {
    flex: 1,
    minHeight: '44px',
    maxHeight: '200px',
    padding: '12px 16px',
    fontSize: '15px',
    fontFamily: 'inherit',
    backgroundColor: obsidianTheme.gray800,
    border: `1px solid ${obsidianTheme.border}`,
    borderRadius: '22px',
    color: obsidianTheme.textPrimary,
    resize: 'none' as const,
    overflowY: 'auto' as const,
    lineHeight: 1.5,
    outline: 'none',
    transition: 'all 0.2s ease',
  } as React.CSSProperties,

  sendButton: {
    width: '44px',
    height: '44px',
    borderRadius: '22px',
    border: 'none',
    backgroundColor: obsidianTheme.accent,
    color: obsidianTheme.white,
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: 'all 0.2s ease',
    flexShrink: 0,
  } as React.CSSProperties,

  sendButtonDisabled: {
    backgroundColor: obsidianTheme.gray700,
    color: obsidianTheme.textTertiary,
    cursor: 'not-allowed',
    opacity: 0.5,
  } as React.CSSProperties,

  sidebarToggle: {
    position: 'fixed' as const,
    top: '16px',
    left: '296px',
    width: '40px',
    height: '40px',
    borderRadius: '12px',
    border: `1px solid ${obsidianTheme.border}`,
    backgroundColor: obsidianTheme.gray800,
    color: obsidianTheme.textPrimary,
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: 'all 0.3s ease',
    zIndex: 1000,
  } as React.CSSProperties,
};

// Add CSS for hover effects and animations
if (!document.getElementById('simple-chat-styles')) {
  const styleSheet = document.createElement('style');
  styleSheet.id = 'simple-chat-styles';
  styleSheet.textContent = `
    .conversation-item:hover {
      background-color: ${obsidianTheme.gray800} !important;
    }
    
    .conversation-item:hover .delete-button {
      opacity: 1 !important;
    }
    
    .new-chat-button:hover {
      background-color: ${obsidianTheme.gray700} !important;
      border-color: ${obsidianTheme.accent} !important;
      transform: scale(1.05);
    }
    
    .send-button:hover:not(:disabled) {
      background-color: ${obsidianTheme.accentHover} !important;
      transform: scale(1.05);
    }
    
    .send-button:active:not(:disabled) {
      transform: scale(0.95);
    }
    
    .image-upload-button:hover:not(:disabled) {
      background-color: ${obsidianTheme.gray700} !important;
      border-color: ${obsidianTheme.accent} !important;
      transform: scale(1.05);
    }
    
    .model-selector-button:hover {
      background-color: ${obsidianTheme.gray700} !important;
      border-color: ${obsidianTheme.accent} !important;
    }
    
    .model-menu-item:hover {
      background-color: ${obsidianTheme.gray700} !important;
    }
    
    .textarea:focus {
      border-color: ${obsidianTheme.accent} !important;
      box-shadow: 0 0 0 3px ${obsidianTheme.accent}20 !important;
    }
    
    .typing-indicator span {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background-color: ${obsidianTheme.textTertiary};
      animation: typing 1.4s infinite;
    }
    
    .typing-indicator span:nth-child(2) {
      animation-delay: 0.2s;
    }
    
    .typing-indicator span:nth-child(3) {
      animation-delay: 0.4s;
    }
    
    @keyframes typing {
      0%, 60%, 100% {
        transform: translateY(0);
        opacity: 0.7;
      }
      30% {
        transform: translateY(-10px);
        opacity: 1;
      }
    }
    
    /* Smooth scrollbar */
    ::-webkit-scrollbar {
      width: 8px;
    }
    
    ::-webkit-scrollbar-track {
      background: ${obsidianTheme.black};
    }
    
    ::-webkit-scrollbar-thumb {
      background: ${obsidianTheme.gray700};
      border-radius: 4px;
    }
    
    ::-webkit-scrollbar-thumb:hover {
      background: ${obsidianTheme.gray600};
    }
  `;
  document.head.appendChild(styleSheet);
}
