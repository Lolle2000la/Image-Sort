using System;
using System.Collections.Generic;
using System.Text;

namespace ImageSort.Actions
{
    public interface IReversibleAction
    {
        void Act();
        void Revert();

        string DisplayName { get; }
    }
}
