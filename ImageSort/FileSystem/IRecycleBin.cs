using System;
using System.Collections.Generic;
using System.Text;

namespace ImageSort.FileSystem
{
    public interface IRecycleBin
    {
        /// <summary>
        /// Sends the file or folder at the given path to the recycle bin.
        /// </summary>
        /// <param name="path">The path to the file or folder that should be send to the recycle bin.</param>
        /// <param name="confirmationNeeded">Should ask for user confirmation before sending to recycle bin.</param>
        /// <returns>The <see cref="IDisposable"/> will, when disposed, restore the file.</returns>
        /// <remarks>
        /// The <see cref="IDisposable"/> handle to the deleted file will throw a <see cref="FileRestorationNotPossibleException"/> 
        /// when it cannot restore the file.
        /// </remarks>
        IDisposable Send(string path, bool confirmationNeeded = false);
    }
}
