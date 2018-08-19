namespace Image_sort.Logic
{
    /// <summary>
    /// Contains all types of event handlers used inside this library.
    /// </summary>
    public static class EventHandlerTypes
    {
        /// <summary>
        /// Handles the folder change event
        /// </summary>
        /// <param name="sender"></param>
        /// <param name="e"></param>
        public delegate void FolderChangedHandler(object sender, FolderChangedEventArgs e);
    }
}