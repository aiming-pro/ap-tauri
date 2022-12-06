const parsePageFromTitle = (title) =>
  title.split('|')[0] !== 'Aiming.Pro ' ? title.split('|')[0] : 'General';

const gameActivity = (status) => {
  const activity = {
    title: status.gameName,
    description: `Current HS: ${status.highScore.toString()}`,
  };

  // Send the activity-update
  window.__TAURI__?.invoke('discordactivity', { activity });
};

const browseActivity = () => {
  // Default activity if window is closed
  const activity = {
    title: 'Browsing',
    description: parsePageFromTitle(document.title),
  };
  // Send the activity-update
  window.__TAURI__?.invoke('discordactivity', { activity });
};

window.addEventListener(
  'DOMContentLoaded',
  () => {
    window.__TAURI__?.invoke('ready');
    // IF GAME PAGE
    if (typeof window.gameVue === 'object') {
      window.__TAURI__?.invoke('gamewindow', { open: true });
    } else {
      // let the controller know and update activity
      browseActivity();
      window.__TAURI__?.invoke('gamewindow', { open: false });
    }

    /* Wait for Game Events to send to the RPC */
    window.addEventListener('game-status-update', (e) => {
      // Prepare the discord template
      gameActivity(e.detail);
    });

    window.addEventListener('game-modal-closed', () => {
      browseActivity();
      window.__TAURI__?.invoke('gamewindow', { open: false });
    });

    // If a game has started
    window.addEventListener('project-started', () => {
      // We don't want to use the regular injection if it's a modal
      if (typeof window.gameVue !== 'object') {
        // Let the controller now that a game has been opened
        window.__TAURI__?.invoke('gamewindow', { open: true });
      }
    });
  },
  false
);
