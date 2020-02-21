using ImageSort.FileSystem;
using System;
using System.Collections.Generic;
using System.Text;

namespace ImageSort.Actions
{
    public class DeleteAction : IReversibleAction
    {
        public DeleteAction(string path, IRecycleBin fileSystem)
        {

        }

        public void Act()
        {
            throw new NotImplementedException();
        }

        public void Revert()
        {
            throw new NotImplementedException();
        }
    }
}
