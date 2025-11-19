/**
 * Design System & Theme
 * 
 * Centralized theme configuration for consistent styling across the application.
 * Provides colors, typography, spacing, shadows, and animations.
 */

export const theme = {
  colors: {
    // Background colors
    background: {
      primary: '#0d1117',      // GitHub dark background
      secondary: '#161b22',     // Slightly lighter
      tertiary: '#1e1e1e',     // Current background
      elevated: '#21262d',     // Elevated surfaces
    },
    
    // Text colors
    text: {
      primary: '#f0f6fc',      // High contrast white
      secondary: '#c9d1d9',    // Medium contrast
      tertiary: '#8b949e',     // Low contrast (timestamps, etc.)
      muted: '#6e7681',        // Very low contrast
    },
    
    // Border colors
    border: {
      primary: '#30363d',      // Main borders
      secondary: '#21262d',    // Subtle borders
      accent: '#1f6feb',       // Accent borders
    },
    
    // Accent colors
    accent: {
      primary: '#1f6feb',      // GitHub blue
      hover: '#388bfd',        // Lighter blue on hover
      active: '#0969da',       // Darker blue when active
    },
    
    // Message colors
    message: {
      user: {
        bg: '#1f6feb',         // User message background
        text: '#ffffff',
        hover: '#388bfd',
      },
      assistant: {
        bg: '#161b22',         // Assistant message background
        text: '#c9d1d9',
        border: '#30363d',
      },
    },
    
    // Status colors
    status: {
      success: '#238636',
      error: '#da3633',
      warning: '#d29922',
      info: '#1f6feb',
    },
    
    // Interactive elements
    interactive: {
      hover: '#21262d',
      active: '#161b22',
      disabled: '#0d1117',
    },
  },
  
  typography: {
    fontFamily: {
      primary: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans", Helvetica, Arial, sans-serif, "Apple Color Emoji"',
      mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace',
    },
    
    fontSize: {
      xs: '0.75rem',      // 12px
      sm: '0.875rem',     // 14px
      base: '0.9375rem',  // 15px
      lg: '1rem',         // 16px
      xl: '1.125rem',     // 18px
      '2xl': '1.25rem',   // 20px
      '3xl': '1.5rem',    // 24px
    },
    
    fontWeight: {
      normal: 400,
      medium: 500,
      semibold: 600,
      bold: 700,
    },
    
    lineHeight: {
      tight: 1.25,
      normal: 1.5,
      relaxed: 1.75,
      loose: 2,
    },
  },
  
  spacing: {
    xs: '0.25rem',   // 4px
    sm: '0.5rem',    // 8px
    md: '0.75rem',   // 12px
    lg: '1rem',      // 16px
    xl: '1.5rem',    // 24px
    '2xl': '2rem',   // 32px
    '3xl': '3rem',   // 48px
  },
  
  borderRadius: {
    sm: '6px',
    md: '8px',
    lg: '12px',
    xl: '16px',
    full: '9999px',
  },
  
  shadows: {
    sm: '0 1px 2px rgba(0, 0, 0, 0.3)',
    md: '0 4px 6px rgba(0, 0, 0, 0.4)',
    lg: '0 10px 15px rgba(0, 0, 0, 0.5)',
    xl: '0 20px 25px rgba(0, 0, 0, 0.6)',
    inner: 'inset 0 2px 4px rgba(0, 0, 0, 0.2)',
  },
  
  transitions: {
    fast: '150ms ease',
    normal: '200ms ease',
    slow: '300ms ease',
    spring: '200ms cubic-bezier(0.4, 0, 0.2, 1)',
  },
  
  zIndex: {
    dropdown: 1000,
    sticky: 1020,
    fixed: 1030,
    modalBackdrop: 1040,
    modal: 1050,
    popover: 1060,
    tooltip: 1070,
  },
} as const;

export type Theme = typeof theme;

