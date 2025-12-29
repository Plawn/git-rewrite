/**
 * Creates a reactive modal state manager.
 * Provides a clean pattern for managing modal open/close state and associated data.
 *
 * @example
 * const editModal = createModalState<CommitInfo>();
 *
 * // Open with data
 * editModal.open(commit);
 *
 * // Access state
 * if (editModal.isOpen) {
 *   console.log(editModal.data);
 * }
 *
 * // Close
 * editModal.close();
 */
export function createModalState<T = undefined>() {
  let isOpen = $state(false);
  let data = $state<T | null>(null);

  return {
    get isOpen() { return isOpen; },
    get data() { return data; },

    open(value?: T) {
      data = value ?? null;
      isOpen = true;
    },

    close() {
      isOpen = false;
      data = null;
    },

    /** Update data without changing open state */
    setData(value: T | null) {
      data = value;
    }
  };
}

/**
 * Creates a modal state with loading support.
 * Useful for modals that fetch data asynchronously.
 *
 * @example
 * const diffModal = createAsyncModalState<CommitDiff>();
 *
 * // Open and load
 * diffModal.open();
 * diffModal.setLoading(true);
 * const data = await fetchDiff();
 * diffModal.setData(data);
 * diffModal.setLoading(false);
 */
export function createAsyncModalState<T>() {
  let isOpen = $state(false);
  let data = $state<T | null>(null);
  let loading = $state(false);

  return {
    get isOpen() { return isOpen; },
    get data() { return data; },
    get loading() { return loading; },

    open() {
      isOpen = true;
      data = null;
      loading = false;
    },

    close() {
      isOpen = false;
      data = null;
      loading = false;
    },

    setData(value: T | null) {
      data = value;
    },

    setLoading(value: boolean) {
      loading = value;
    }
  };
}
