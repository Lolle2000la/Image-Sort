using ImageSort.FileSystem;
using System;
using System.Collections.Generic;
using System.Text;

namespace ImageSort.WPF.FileSystem
{
    class RecycleBin : IRecycleBin
    {
        public IDisposable Send(string path, bool confirmationNeeded = false)
        {
            throw new NotImplementedException();
        }
    }
}
