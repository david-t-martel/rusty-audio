/**
 * Secure Token Storage using IndexedDB with Web Crypto API encryption
 *
 * Provides secure storage for OAuth tokens with automatic encryption/decryption
 */

class SecureTokenStorage {
  constructor() {
    this.dbName = 'rusty-audio-auth';
    this.storeName = 'tokens';
    this.version = 1;
    this.db = null;
    this.encryptionKey = null;
  }

  /**
   * Initialize the IndexedDB database
   */
  async init() {
    if (this.db) return;

    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, this.version);

      request.onerror = () => reject(new Error('Failed to open database'));

      request.onsuccess = (event) => {
        this.db = event.target.result;
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = event.target.result;

        if (!db.objectStoreNames.contains(this.storeName)) {
          db.createObjectStore(this.storeName, { keyPath: 'key' });
        }
      };
    });
  }

  /**
   * Generate or retrieve encryption key from Web Crypto API
   */
  async getEncryptionKey() {
    if (this.encryptionKey) return this.encryptionKey;

    // Check if key exists in storage
    const storedKey = await this._getFromDB('encryption_key');

    if (storedKey) {
      // Import existing key
      this.encryptionKey = await crypto.subtle.importKey(
        'jwk',
        storedKey.value,
        { name: 'AES-GCM', length: 256 },
        true,
        ['encrypt', 'decrypt']
      );
    } else {
      // Generate new key
      this.encryptionKey = await crypto.subtle.generateKey(
        { name: 'AES-GCM', length: 256 },
        true,
        ['encrypt', 'decrypt']
      );

      // Export and store the key
      const exportedKey = await crypto.subtle.exportKey('jwk', this.encryptionKey);
      await this._saveToDB('encryption_key', exportedKey);
    }

    return this.encryptionKey;
  }

  /**
   * Encrypt data using AES-GCM
   */
  async encrypt(data) {
    const key = await this.getEncryptionKey();
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encoder = new TextEncoder();
    const encodedData = encoder.encode(JSON.stringify(data));

    const encryptedData = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      key,
      encodedData
    );

    // Combine IV and encrypted data
    const combined = new Uint8Array(iv.length + encryptedData.byteLength);
    combined.set(iv, 0);
    combined.set(new Uint8Array(encryptedData), iv.length);

    // Convert to base64 for storage
    return btoa(String.fromCharCode(...combined));
  }

  /**
   * Decrypt data using AES-GCM
   */
  async decrypt(encryptedBase64) {
    const key = await this.getEncryptionKey();
    const combined = Uint8Array.from(atob(encryptedBase64), c => c.charCodeAt(0));

    // Extract IV and encrypted data
    const iv = combined.slice(0, 12);
    const encryptedData = combined.slice(12);

    const decryptedData = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv },
      key,
      encryptedData
    );

    const decoder = new TextDecoder();
    return JSON.parse(decoder.decode(decryptedData));
  }

  /**
   * Store tokens securely
   */
  async storeTokens(accessToken, refreshToken, expiresIn) {
    await this.init();

    const tokenData = {
      accessToken,
      refreshToken,
      expiresAt: Date.now() + (expiresIn * 1000),
      createdAt: Date.now()
    };

    const encrypted = await this.encrypt(tokenData);
    await this._saveToDB('auth_tokens', encrypted);
  }

  /**
   * Get access token (returns null if expired)
   */
  async getAccessToken() {
    await this.init();

    const encrypted = await this._getFromDB('auth_tokens');
    if (!encrypted || !encrypted.value) return null;

    try {
      const tokenData = await this.decrypt(encrypted.value);

      // Check if token is expired
      if (Date.now() >= tokenData.expiresAt) {
        console.log('Access token expired');
        return null;
      }

      return tokenData.accessToken;
    } catch (error) {
      console.error('Failed to decrypt access token:', error);
      return null;
    }
  }

  /**
   * Get refresh token
   */
  async getRefreshToken() {
    await this.init();

    const encrypted = await this._getFromDB('auth_tokens');
    if (!encrypted || !encrypted.value) return null;

    try {
      const tokenData = await this.decrypt(encrypted.value);
      return tokenData.refreshToken;
    } catch (error) {
      console.error('Failed to decrypt refresh token:', error);
      return null;
    }
  }

  /**
   * Check if token is valid (not expired)
   */
  async isTokenValid() {
    await this.init();

    const encrypted = await this._getFromDB('auth_tokens');
    if (!encrypted || !encrypted.value) return false;

    try {
      const tokenData = await this.decrypt(encrypted.value);
      return Date.now() < tokenData.expiresAt;
    } catch (error) {
      return false;
    }
  }

  /**
   * Get time until token expiration (in milliseconds)
   */
  async getTimeUntilExpiration() {
    await this.init();

    const encrypted = await this._getFromDB('auth_tokens');
    if (!encrypted || !encrypted.value) return 0;

    try {
      const tokenData = await this.decrypt(encrypted.value);
      return Math.max(0, tokenData.expiresAt - Date.now());
    } catch (error) {
      return 0;
    }
  }

  /**
   * Store user information
   */
  async storeUserInfo(userInfo) {
    await this.init();
    const encrypted = await this.encrypt(userInfo);
    await this._saveToDB('user_info', encrypted);
  }

  /**
   * Get user information
   */
  async getUserInfo() {
    await this.init();

    const encrypted = await this._getFromDB('user_info');
    if (!encrypted || !encrypted.value) return null;

    try {
      return await this.decrypt(encrypted.value);
    } catch (error) {
      console.error('Failed to decrypt user info:', error);
      return null;
    }
  }

  /**
   * Clear all tokens and user data
   */
  async clearTokens() {
    await this.init();

    return new Promise((resolve, reject) => {
      const transaction = this.db.transaction([this.storeName], 'readwrite');
      const store = transaction.objectStore(this.storeName);

      // Delete auth tokens and user info, but keep encryption key
      store.delete('auth_tokens');
      store.delete('user_info');

      transaction.oncomplete = () => resolve();
      transaction.onerror = () => reject(new Error('Failed to clear tokens'));
    });
  }

  /**
   * Clear everything including encryption key (complete reset)
   */
  async clearAll() {
    await this.init();

    return new Promise((resolve, reject) => {
      const transaction = this.db.transaction([this.storeName], 'readwrite');
      const store = transaction.objectStore(this.storeName);

      store.clear();

      transaction.oncomplete = () => {
        this.encryptionKey = null;
        resolve();
      };
      transaction.onerror = () => reject(new Error('Failed to clear all data'));
    });
  }

  /**
   * Internal method to save to IndexedDB
   */
  async _saveToDB(key, value) {
    return new Promise((resolve, reject) => {
      const transaction = this.db.transaction([this.storeName], 'readwrite');
      const store = transaction.objectStore(this.storeName);

      const request = store.put({ key, value });

      request.onsuccess = () => resolve();
      request.onerror = () => reject(new Error(`Failed to save ${key}`));
    });
  }

  /**
   * Internal method to get from IndexedDB
   */
  async _getFromDB(key) {
    return new Promise((resolve, reject) => {
      const transaction = this.db.transaction([this.storeName], 'readonly');
      const store = transaction.objectStore(this.storeName);

      const request = store.get(key);

      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(new Error(`Failed to get ${key}`));
    });
  }

  /**
   * Get storage statistics
   */
  async getStats() {
    await this.init();

    const tokens = await this._getFromDB('auth_tokens');
    const userInfo = await this._getFromDB('user_info');

    return {
      hasTokens: !!tokens,
      hasUserInfo: !!userInfo,
      isTokenValid: await this.isTokenValid(),
      timeUntilExpiration: await this.getTimeUntilExpiration()
    };
  }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
  module.exports = SecureTokenStorage;
}
